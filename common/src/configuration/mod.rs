#![allow(clippy::needless_return)]
#![deny(clippy::implicit_return)]


mod validation;

use std::env;
use std::ffi::OsString;
use std::fs::create_dir_all;
use std::io::ErrorKind;
use std::path::Path;
use std::string::ToString;

use anyhow::anyhow;
use anyhow::Error;
use anyhow::Result;

use serde::Deserialize;

use crate::registry::get_nbi_value;
use validation::validate_log_directory;

const PROCESSES_VALUE_NAME: &str = "Processes";
const TRACE_NAME_VALUE_NAME: &str = "TraceName";
const HOOK_DLL_NAME_VALUE_NAME: &str = "HookDllName";
const LOG_DIRECTORY_VALUE_NAME: &str = "LogDirectory";

#[cfg(target_arch = "x86_64")]
const DEFAULT_TRACE_NAME: &str = "NoBlockInput";
#[cfg(target_arch = "x86")]
const DEFAULT_TRACE_NAME: &str = "NoBlockInput_x86";

#[derive(Debug)]
#[allow(dead_code)]
pub struct FullConfig
{
	x64: InjectorConfig,
	x86: InjectorConfig
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct InjectorConfig
{
	pub hook_dll_path: OsString,
	pub trace_name: String,
	pub processes: Vec<String>,
	pub log_directory: Result<OsString, Error>
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct RawFullConfig
{
	x64: RawInjectorConfig,
	x86: RawInjectorConfig
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct RawInjectorConfig
{
	pub hook_dll_name: Option<String>,
	pub trace_name: Option<String>,
	pub processes: Option<Vec<String>>,
	pub log_directory: Option<String>
}

impl InjectorConfig
{
	pub fn get_configuration() -> Result<Self, Error>
	{
		let processes: Result<Vec<String>, std::io::Error> = get_nbi_value(PROCESSES_VALUE_NAME);
		let processes = match processes
		{
			Ok(processes) =>
			{
				if processes[0] == *"" { return Err(anyhow!("The processes entry in the registry configuration is empty.")); }
				processes
			}
			Err(err) =>
			{
				if err.kind().eq(&ErrorKind::NotFound)
				{
					return Err(anyhow!("There was an error trying to retrieve the list of processes to hook from the registry: {err}"));
				}
				return Err(anyhow!("No processes were found in the registry configuration."));
			}
		};

		let trace_name: Result<String, std::io::Error> = get_nbi_value(TRACE_NAME_VALUE_NAME);
		let trace_name = match trace_name
		{
			Ok(trace_name) =>
			{
				match trace_name.is_empty()
				{
					true => DEFAULT_TRACE_NAME.to_string(),
					false => trace_name
				}
			}
			Err(err) =>
			{
				if err.kind().eq(&ErrorKind::NotFound)
				{
					return Err(anyhow!("There was an error trying to retrieve the trace name from the registry: {err}"));
				}
				DEFAULT_TRACE_NAME.to_string()
			}
		};

		let hook_dll_name: Result<String, std::io::Error> = get_nbi_value(HOOK_DLL_NAME_VALUE_NAME);
		let hook_dll_path = match hook_dll_name
		{
			Ok(hook_dll_name) =>
			{
				match hook_dll_name.is_empty()
				{
					true => get_hook_dll_path(&None)?,
					false => get_hook_dll_path(&Some(hook_dll_name))?
				}
			}
			Err(err) =>
			{
				if err.kind().eq(&ErrorKind::NotFound)
				{
					return Err(anyhow!("There was an error trying to retrieve the hook DLL name from the registry: {err}"));
				}
				get_hook_dll_path(&None)?
			}
		};

		let log_directory: Result<OsString, std::io::Error> = get_nbi_value(LOG_DIRECTORY_VALUE_NAME);
		let log_directory = match log_directory
		{
			Ok(log_directory) => Ok(log_directory),
			Err(err) => Err(Some(err))
		};
		let log_directory = validate_log_directory(log_directory);
		let log_directory = match log_directory
		{
			Ok(log_directory) => create_log_dir(log_directory),
			Err(_) => log_directory
		};

		return Ok(InjectorConfig { hook_dll_path, processes, trace_name, log_directory });
	}
}

fn get_hook_dll_path(hook_dll_name: &Option<String>) -> Result<OsString, Error>
{
	let name = match hook_dll_name
	{
		None => { get_dll_path_from_exe() }
		Some(name) =>
		{
			if name.is_empty()
			{
				let hook_dll_path = get_dll_path_from_exe()?;
				return Ok(hook_dll_path);
			}
			return Ok(OsString::from(name));
		}
	};
	return name;
}

fn create_log_dir(log_directory: OsString) -> Result<OsString, Error>
{
	let directory_creation_result = create_dir_all(&log_directory);
	return match directory_creation_result
	{
		Ok(_) => { Ok(log_directory) }
		Err(err) =>
		{
			match err.kind()
			{
				ErrorKind::AlreadyExists => Ok(log_directory),
				_ => Err(anyhow!("Log directory could not be created: {err}"))
			}
		}
	};
}

fn get_dll_path_from_exe() -> Result<OsString, Error>
{
	let exe_path = get_exe_path()?;
	let exe_path = Path::new(&exe_path);
	return Ok(OsString::from(exe_path.with_extension("dll")));
}

fn get_exe_path() -> Result<OsString, Error>
{
	let exe_path = env::current_exe();
	return match exe_path
	{
		Ok(exe_path) => Ok(exe_path.into_os_string()),
		Err(err) => { return Err(anyhow!("Couldn't get the path of the this executable: {err}")); }
	};
}