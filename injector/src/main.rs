#![cfg_attr(all(target_os = "windows", not(debug_assertions)),
windows_subsystem = "windows")]
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;
use dll_syringe::process::{BorrowedProcessModule, OwnedProcess};

use dll_syringe::Syringe;

use ferrisetw::EventRecord;
use ferrisetw::parser::Parser;
use ferrisetw::provider::Provider;
use ferrisetw::schema_locator::SchemaLocator;
use ferrisetw::trace::{stop_trace_by_name, UserTrace};
use ferrisetw::trace::TraceTrait;
use ferrisetw::trace::TraceError;
use ferrisetw::native::TraceHandle;
use ferrisetw::native::EvntraceNativeError;
use windows::Win32::System::Diagnostics::Debug::{DebugActiveProcess, DebugActiveProcessStop};

const MICROSOFT_WINDOWS_KERNEL_PROCESS_PROVIDER: &str = "22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716";
const HOOK_DLL_NAME: &str = "noblock_input_hook.dll";

fn main()
{
	let hook_dll_path = get_hook_dll_path();
	let client_processes = OwnedProcess::find_all_by_name("ScreenConnect.WindowsClient");
	for client_process in client_processes
	{
		let syringe: Syringe = Syringe::for_process(client_process);
		let _payload: BorrowedProcessModule = syringe.inject(&hook_dll_path).unwrap();
	}

	let process_provider = Provider::by_guid(MICROSOFT_WINDOWS_KERNEL_PROCESS_PROVIDER).add_callback(process_callback).build();

	let (_user_trace, handle) = get_result(process_provider);

	std::thread::spawn(move || {
		let status = UserTrace::process_from_handle(handle);
		// This code will be executed when the trace stops. Examples:
		// * when it is dropped
		// * when it is manually stopped (either by user_trace.stop, or by the `logman stop -ets MyTrace` command)
		println!("Trace ended with status {:?}", status);
	});

	ctrlc::set_handler(move || { std::process::exit(0); }).expect("Error setting Ctrl-C handler");

	sleep(Duration::from_secs(5));

	loop
	{
		sleep(Duration::from_millis(10));
	}
}


fn process_callback(record: &EventRecord, schema_locator: &SchemaLocator)
{
	let hook_dll_path = get_hook_dll_path();
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
					if file_name != "ScreenConnect.WindowsClient.exe" { return; }
					// if file_name != "notepad.exe" { return; }
					unsafe
						{
							std::thread::spawn(move ||
							{
								DebugActiveProcess(process_id).unwrap();
								sleep(Duration::from_millis(1000));
								DebugActiveProcessStop(process_id).unwrap();
							});
							let owned_sc_client = OwnedProcess::from_pid(process_id).unwrap();
							let syringe: Syringe = Syringe::for_process(owned_sc_client);
							let _payload: BorrowedProcessModule = syringe.inject(hook_dll_path).unwrap();
						}
				}
			},
		Err(err) => println!("Error {:?}", err)
	}
}

fn get_result(provider: Provider) -> (UserTrace, TraceHandle)
{
	let th_result = UserTrace::new().named(String::from("MyTrace")).enable(provider).start();
	match th_result
	{
		Ok((ref _user_trace, _trace_handle)) => { return th_result.unwrap(); },
		Err(TraceError::EtwNativeError(err)) => match err
		{
			EvntraceNativeError::AlreadyExist =>
				{
					let provider = Provider::by_guid(MICROSOFT_WINDOWS_KERNEL_PROCESS_PROVIDER).add_callback(process_callback).build();
					stop_trace_by_name("MyTrace").expect("Couldn't stop existing trace.");
					return UserTrace::new().named(String::from("MyTrace")).enable(provider).start().unwrap();
				},
			_ => panic!("Couldn't start a trace: {:?}", err)
		},
		Err(err) => panic!("Couldn't start a trace: {:?}", err)
	};
}

fn get_hook_dll_path() -> String
{
	let hook_dll_path: PathBuf = std::env::current_exe().unwrap();
	let hook_dll_path: &Path = hook_dll_path.as_path().parent().unwrap();
	let hook_dll_path: PathBuf = hook_dll_path.join(HOOK_DLL_NAME);
	return String::from(hook_dll_path.to_str().unwrap());
}