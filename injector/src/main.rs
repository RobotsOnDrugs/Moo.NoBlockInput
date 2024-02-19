#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]
#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]


use std::collections::HashSet;
use std::path::Path;
use std::process::exit;
use std::string::ToString;
use std::sync::mpsc;
use std::thread::sleep;
use std::time::Duration;

use dll_syringe::Syringe;
use dll_syringe::process::BorrowedProcessModule;
use dll_syringe::process::OwnedProcess;
use dll_syringe::process::Process;

use ferrisetw::EventRecord;
use ferrisetw::parser::Parser;
use ferrisetw::provider::EventFilter;
use ferrisetw::provider::Provider;
use ferrisetw::schema_locator::SchemaLocator;
use ferrisetw::trace::RealTimeTraceTrait;
use ferrisetw::trace::stop_trace_by_name;
use ferrisetw::trace::TraceTrait;
use ferrisetw::trace::UserTrace;

use log::error;
use log::info;
use log::warn;

use simplelog::CombinedLogger;

use windows::Win32::System::Diagnostics::Debug::DebugActiveProcess;
use windows::Win32::System::Diagnostics::Debug::DebugActiveProcessStop;

use noblock_input_common::configuration::InjectorConfig;
use noblock_input_common::logging::create_logger;

const MICROSOFT_WINDOWS_KERNEL_PROCESS_PROVIDER: &str = "22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716";
const IMAGE_LOAD: u16 = 5;
const IMAGE_UNLOAD: u16 = 6;

fn main()
{
	let configuration = InjectorConfig::try_new(None, None);
	let configuration = match configuration
	{
		Ok(configuration) => { configuration }
		Err(err) => { println!("Could not load configuration: {err}"); exit(-1); }
	};
	let logger = create_logger(std::env::current_exe().unwrap(), configuration.log_directory);
	CombinedLogger::init(logger.loggers).unwrap();
	if logger.log_file_path.is_err() { warn!("Could not create log file: {}", logger.log_file_path.unwrap_err()) }
	info!("Starting!");

	let hook_dll_path = configuration.hook_dll_path.to_owned();

	let mut all_client_processes: HashSet<OwnedProcess> = HashSet::new();
	for proc_name in &configuration.processes
	{
		let client_processes = OwnedProcess::find_all_by_name(proc_name);
		for client_process in client_processes
		{
			all_client_processes.insert(client_process);
		}
	}

	let mut hooked_processes: HashSet<u32> = HashSet::new();
	for client_process in all_client_processes
	{
		let pid = client_process.pid().unwrap();
		let syringe: Syringe = Syringe::for_process(client_process);
		hooked_processes.insert(u32::from(pid));
		let _payload: BorrowedProcessModule = syringe.inject(&hook_dll_path).unwrap();
		let pid_result = syringe.process().pid();
		let name_result = syringe.process().base_name();
		let process_name = name_result.unwrap();
		let process_name = process_name.to_string_lossy();
		info!("Hooked existing process {} (PID {:?})", process_name, pid_result.unwrap());
	}

	// Using a cloned hooked_process for use in the callback and a channel to communicate to the Ctrl-C handler from the callback
	// Seems kind of messy, but it's necessary to move variables into those threads (probably)
	let mut hooked_processes = hooked_processes.clone();
	let (ctrl_c_tx, ctrl_c_rx) = mpsc::channel::<HashSet<u32>>();
	ctrl_c_tx.send(hooked_processes.to_owned()).unwrap();

	let process_callback = move |record: &EventRecord, schema_locator: &SchemaLocator|
	{
		match schema_locator.event_schema(record)
		{
			Ok(schema) =>
			{
				let parser = Parser::create(record, &schema);
				let process_id: u32 = parser.try_parse("ProcessID").unwrap();
				let image_name: String = parser.try_parse("ImageName").unwrap();
				let file_name = Path::new(&image_name).file_name().unwrap();
				let file_name = file_name.to_string_lossy().to_string();
				if !configuration.processes.contains(&file_name) { return; }
				if record.event_id() == IMAGE_UNLOAD
				{
					hooked_processes.remove(&process_id);
					info!("Process exited: {} (PID {})", file_name, process_id);
					ctrl_c_tx.send(hooked_processes.clone()).unwrap();
					return;
				}
				unsafe
					{
						std::thread::spawn(move ||
							{
								let attachment_result = DebugActiveProcess(process_id);
								// Sometimes the process is crashy and dies very quickly and it's better to just ignore it and move on
								if attachment_result.is_err() { return; }
								sleep(Duration::from_millis(1000));
								DebugActiveProcessStop(process_id).unwrap();
							});
					}
				let owned_process_result = OwnedProcess::from_pid(process_id);
				// Same here: if the process goes bye-bye when we want to own it or inject into it, we march on
				if owned_process_result.is_err() { return; }
				let owned_process = owned_process_result.unwrap();
				let syringe: Syringe = Syringe::for_process(owned_process);
				let dll_path = configuration.hook_dll_path.to_owned();
				let payload_result = syringe.inject(dll_path);
				if payload_result.is_err()
				{
					warn!("Could not inject into {} {:?}", &file_name, payload_result.unwrap_err());
					return;
				}
				info!("Hooked in callback: {file_name} (PID {process_id})");
				hooked_processes.insert(process_id);
				ctrl_c_tx.send(hooked_processes.clone()).unwrap();
			}
			Err(err) => error!("Error {:?}", err)
		}
	};

	let process_provider = Provider::by_guid(MICROSOFT_WINDOWS_KERNEL_PROCESS_PROVIDER)
		.add_filter(EventFilter::ByEventIds(vec![IMAGE_LOAD, IMAGE_UNLOAD]))
		.add_callback(process_callback)
		.build();

	let trace_name = configuration.trace_name;
	let _ = stop_trace_by_name(&trace_name);

	let (user_trace, handle) = UserTrace::new().named(String::from(&trace_name)).enable(process_provider).start().unwrap();
	let name = user_trace.trace_name();

	let trace_name = trace_name.to_owned();
	let dll_path = hook_dll_path.to_owned();

	let do_cleanup = move ||
	{
		info!("Shutting down...");
		let trace_name = &trace_name;
		let dll_path = &dll_path;
		let stopped_trace_result = stop_trace_by_name(trace_name.as_str());
		let err_code = match stopped_trace_result
		{
			Ok(()) => 0,
			Err(_) => -1
		};
		let mut hooked_processes: HashSet<u32> = HashSet::new();
		loop
		{
			let current_hooked_processes_result = ctrl_c_rx.try_recv();
			match current_hooked_processes_result
			{
				Ok(current_hooked_processes) => { hooked_processes = current_hooked_processes; }
				Err(_) => { break; }
			}
		}
		for pid in hooked_processes
		{
			let process = OwnedProcess::from_pid(pid);
			// ScreenConnect service process exit events are apparently not captured, so the workaround is to just ignore PIDs that are gone
			if process.is_err()
			{
				warn!("PID {pid} exited without the image unload event being captured (detected during cleanup).");
				continue;
			}
			let process = process.unwrap();
			let module = process.find_module_by_path(dll_path).unwrap().unwrap();
			let syringe: Syringe = Syringe::for_process(process);
			let module = syringe.eject(module.borrowed());
			let process_name = &syringe.process().base_name().unwrap();
			let process_name = process_name.to_string_lossy();
			match module
			{
				Ok(_) =>
				{
					info!("Successfully ejected module from {process_name} (PID {pid}).");
				},
			Err(err) =>
				{
					error!("Failed to eject module from {process_name} (PID {pid}): {:?}", err);
				}
			}
		}
		exit(err_code);
	};
	ctrlc::set_handler(do_cleanup).expect("Error setting Ctrl-C handler.");

	std::thread::spawn(move ||
		{
			let status = UserTrace::process_from_handle(handle);
			// This code will be executed when the trace stops. Examples:
			// * when it is dropped
			// * when it is manually stopped (either by user_trace.stop, or by the `logman stop -ets MyTrace` command)
			match status
			{
				Ok(_) => { info!("Trace {} was successfully terminated.", &name.to_string_lossy()); }
				Err(err) => { info!("Trace {} could not be terminated: {err:?}", &name.to_string_lossy()); }
			}
		});

	loop { sleep(Duration::from_millis(100)); }
}