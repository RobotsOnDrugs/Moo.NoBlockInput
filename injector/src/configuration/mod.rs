#![allow(clippy::needless_return)]
#![deny(clippy::implicit_return)]

use std::env;
use std::ffi::OsString;
use std::fs::File;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use serde::Deserialize;

use winreg::RegKey;
use winreg::enums::HKEY_LOCAL_MACHINE;

const NBI_REGISTRY_PATH: &str = r#"SOFTWARE\Moo\NoBlockInput"#;

#[cfg(target_arch = "x86_64")]
const DEFAULT_TRACE_NAME: &str = "NoBlockInput";
#[cfg(target_arch = "x86")]
const DEFAULT_TRACE_NAME: &str = "NoBlockInput_x86";

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct FullConfig
{
	x64: InjectorConfig,
	x86: InjectorConfig
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct InjectorConfig
{
	pub hook_dll_path: String,
	pub trace_name: String,
	pub processes: Vec<String>
}

impl InjectorConfig
{
	// Default is to use the registry and fall back to the config file if there is an error
	// Explicitly using the registry will not fall back to the config file
	pub fn try_new(use_registry: Option<bool>, config_file_path: Option<OsString>) -> Option<Self>
	{
		return match use_registry
		{
			None =>
			{
				let config = Self::read_registry();
				match config
				{
					Ok(config) => { Some(config) }
					Err(_) => { Self::read_configuration_file(None) }
				}
			}
			Some(true) => { Self::read_registry().ok() }
			Some(false) => { Self::read_configuration_file(config_file_path) }
		};
	}

	fn read_configuration_file(file_path: Option<OsString>) -> Option<Self>
	{
		let file_path = match file_path
		{
			None => { OsString::from("injector.toml") }
			Some(file_path) => { file_path }
		};
		let toml_file = File::open(file_path);
		let mut toml_file = match toml_file
		{
			Ok(file) => file,
			// println!("Couldn't open the injector configuration file: {err}");
			Err(_) => { return None; }
		};
		let mut config_toml = String::new();
		let toml_read_result = toml_file.read_to_string(&mut config_toml);
		match toml_read_result
		{
			Ok(_) => { },
			// println!("Couldn't read the injector configuration file: {err}");
			Err(_) => { return None; }
		};
		let toml_str = config_toml.as_str();
		let injector_config: Result<FullConfig, toml::de::Error> = toml::from_str(toml_str);
		let injector_config = match injector_config
		{
			Ok(config) => config,
			// println!("Couldn't parse the injector configuration file: {err}");
			Err(_) => { return None; }
		};

		#[cfg(target_arch = "x86_64")]
		let configuration = injector_config.x64;
		#[cfg(target_arch = "x86")]
		let configuration = injector_config.x86;

		return Some(configuration);
	}

	fn read_registry() -> Result<Self, Error>
	{
		let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
		let nbi_key = hklm.open_subkey(NBI_REGISTRY_PATH)?;
		let trace_name: std::io::Result<String> = nbi_key.get_value("TraceName");
		let trace_name = match trace_name
		{
			Ok(name) =>
			{
				if name.is_empty() { String::from(DEFAULT_TRACE_NAME) }
				else { name }
			}
			Err(err) =>
			{
				match err.kind()
				{
					ErrorKind::NotFound => { String::from(DEFAULT_TRACE_NAME) }
					_ => { return Err(err); }
				}
			}
		};
		let processes: Vec<String> = nbi_key.get_value("Processes")?;
		let hook_dll_name: String = nbi_key.get_value("HookDllName")?;
		let hook_dll_name = match hook_dll_name.is_empty()
		{
			true => { None }
			false => { Some(hook_dll_name) }
		};
		let hook_dll_path = get_hook_dll_path(hook_dll_name)?;
		return Ok(InjectorConfig { hook_dll_path, processes, trace_name });
	}
}

fn get_hook_dll_path(hook_dll_name: Option<String>) -> Result<String, Error>
{
	let exe_path: PathBuf = env::current_exe()?;
	let cwd = Path::new(&exe_path).parent().unwrap();
	let name = match hook_dll_name
	{
		Some(name) => String::from(cwd.join(name).to_str().unwrap()),
		None => String::from(exe_path.with_extension("dll").to_str().unwrap())
	};
	return Ok(name);
}