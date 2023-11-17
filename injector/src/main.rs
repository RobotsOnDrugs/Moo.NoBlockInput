#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]

use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use std::io::Read;
use std::process::exit;
use std::string::ToString;

use dll_syringe::Syringe;
use dll_syringe::process::BorrowedProcessModule;
use dll_syringe::process::Process;
use dll_syringe::process::OwnedProcess;

use serde::Deserialize;

use ferrisetw::EventRecord;
use ferrisetw::native::TraceHandle;
use ferrisetw::native::EvntraceNativeError;
use ferrisetw::parser::Parser;
use ferrisetw::provider::Provider;
use ferrisetw::schema_locator::SchemaLocator;
use ferrisetw::trace::RealTimeTraceTrait;
use ferrisetw::trace::stop_trace_by_name;
use ferrisetw::trace::TraceError;
use ferrisetw::trace::TraceTrait;
use ferrisetw::trace::UserTrace;

use windows::Win32::System::Diagnostics::Debug::DebugActiveProcess;
use windows::Win32::System::Diagnostics::Debug::DebugActiveProcessStop;

const MICROSOFT_WINDOWS_KERNEL_PROCESS_PROVIDER: &str = "22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716";

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FullConfig
{
	x64: InjectorConfig,
	x86: InjectorConfig
}

#[derive(Debug, Deserialize)]
struct InjectorConfig
{
	hook_dll_name: Option<String>,
	trace_name: String,
	processes: Vec<String>
}

fn main()
{
	let toml_file = File::open("injector.toml");
	let mut toml_file = match toml_file
	{
		Ok(file) => file,
		Err(err) => { println!("Couldn't open the injector configuration file: {err}"); return; }
	};
	let mut config_toml = String::new();
	let toml_read_result = toml_file.read_to_string(&mut config_toml);
	match toml_read_result
	{
		Ok(_) => { },
		Err(err) => { println!("Couldn't read the injector configuration file: {err}"); return; }
	};
	let toml_str = config_toml.as_str();
	let injector_config: Result<FullConfig, toml::de::Error> = toml::from_str(toml_str);
	let injector_config = match injector_config
	{
		Ok(config) => config,
		Err(err) => { println!("Couldn't parse the injector configuration file: {err}"); return; }
	};

	#[cfg(target_arch = "x86_64")]
	let configuration = injector_config.x64;
	#[cfg(target_arch = "x86")]
	let configuration = injector_config.x86;

	let hook_dll_name = configuration.hook_dll_name;
	// Changed the signature to set up for future work
	let hook_dll_path = get_hook_dll_path(&hook_dll_name);
	env::set_var("NOBLOCKINPUT_DLL_PATH", &hook_dll_path);
	env::set_var("NOBLOCKINPUT_TRACE_NAME", configuration.trace_name);

	let mut all_client_processes: HashSet<OwnedProcess> = HashSet::new();
	for proc_name in &configuration.processes
	{
		let client_processes = OwnedProcess::find_all_by_name(proc_name);
		for client_process in client_processes
		{
			all_client_processes.insert(client_process);
		}
	}
	env::set_var("NOBLOCKINPUT_PROCESSES", configuration.processes.join(";"));

	let all_client_processes = all_client_processes;
	for client_process in all_client_processes
	{
		let syringe: Syringe = Syringe::for_process(client_process);
		let _payload: BorrowedProcessModule = syringe.inject(hook_dll_path.as_str()).unwrap();
		let pid_result = syringe.process().pid();
		let name_result = syringe.process().base_name();
		println!("{:?} {:?} [hooked existing]", pid_result.unwrap(), name_result.unwrap());
	}

	let process_provider = Provider::by_guid(MICROSOFT_WINDOWS_KERNEL_PROCESS_PROVIDER).add_callback(process_callback).build();

	let (user_trace, handle) = get_result(process_provider);
	let name = user_trace.trace_name();
	println!("Trace name: {:?}", name);

	std::thread::spawn(move ||
	{
		let status = UserTrace::process_from_handle(handle);
		// This code will be executed when the trace stops. Examples:
		// * when it is dropped
		// * when it is manually stopped (either by user_trace.stop, or by the `logman stop -ets MyTrace` command)
		println!("Trace ended with status {:?}", status);
	});

	ctrlc::set_handler(move ||
	{
		let trace_name = env::var("NOBLOCKINPUT_TRACE_NAME").expect("TRACE_NAME wasn't previously set.");
		let stopped_trace_result = stop_trace_by_name(trace_name.as_str());
		let err_code = match stopped_trace_result
		{
			Ok(()) => 0,
			Err(_) => -1
		};
		exit(err_code);
	}).expect("Error setting Ctrl-C handler");

	loop { sleep(Duration::from_millis(10)); }
}

fn get_result(provider: Provider) -> (UserTrace, TraceHandle)
{
	let trace_name = env::var("NOBLOCKINPUT_TRACE_NAME").expect("TRACE_NAME wasn't previously set.");
	let th_result = UserTrace::new().named(String::from(&trace_name)).enable(provider).start();
	match th_result
	{
		Ok((ref _user_trace, _trace_handle)) => { return th_result.unwrap(); },
		Err(TraceError::EtwNativeError(err)) => match err
		{
			EvntraceNativeError::AlreadyExist =>
				{
					let provider = Provider::by_guid(MICROSOFT_WINDOWS_KERNEL_PROCESS_PROVIDER).add_callback(process_callback).build();
					stop_trace_by_name(&trace_name).expect("Couldn't stop existing trace.");
					return UserTrace::new().named(String::from(&trace_name)).enable(provider).start().unwrap();
				},
			_ => panic!("Couldn't start a trace: {:?}", err)
		},
		Err(err) => panic!("Couldn't start a trace: {:?}", err)
	};
}

fn process_callback(record: &EventRecord, schema_locator: &SchemaLocator)
{
	let hook_targets = env::var("NOBLOCKINPUT_PROCESSES").expect("HOOK_TARGETS was not previously set");
	let hook_targets: Vec<&str> = hook_targets.split(';').collect();
	match schema_locator.event_schema(record)
	{
		Ok(schema) =>
		{
			let event_id = record.event_id();
			if event_id == 1
			{
				let parser = Parser::create(record, &schema);
				let process_id: u32 = parser.try_parse("ProcessID").unwrap();
				let image_name: String = parser.try_parse("ImageName").unwrap();
				let file_name = Path::new(&image_name).file_name().unwrap();
				let file_name = file_name.to_str().unwrap().to_string();
				if !hook_targets.contains(&file_name.as_str()) { return; }
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
				// Same here: if the process goes bye-bye when we want to own it or inject into it, we march on
				let owned_sc_client_result = OwnedProcess::from_pid(process_id);
				if owned_sc_client_result.is_err() { return; }
				let owned_sc_client = owned_sc_client_result.unwrap();
				let syringe: Syringe = Syringe::for_process(owned_sc_client);
				let dll_path = env::var("NOBLOCKINPUT_DLL_PATH").expect("DLL_PATH was not previously set");
				let payload_result = syringe.inject(dll_path);
				if payload_result.is_err() {
					println!("{:?}", payload_result.err().unwrap());
					return;
				}
				println!("{} {} [hooked]", process_id, file_name);
			}
		},
		Err(err) => println!("Error {:?}", err)
	}
}

fn get_hook_dll_path(hook_dll_name: &Option<String>) -> String
{
	let exe_path: PathBuf = env::current_exe().unwrap();
	let cwd = Path::new(&exe_path).parent().unwrap();
	return match hook_dll_name
	{
		Some(name) => String::from(cwd.join(name).to_str().unwrap()),
		None => String::from(exe_path.with_extension("dll").to_str().unwrap())
	}
}