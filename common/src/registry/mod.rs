#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use std::ffi::OsStr;
use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;
use winreg::types::{FromRegValue, ToRegValue};

pub const NBI_REGISTRY_PATH: &str = r#"SOFTWARE\Moo\NoBlockInput"#;

pub fn get_nbi_value<T: FromRegValue>(value_name: &str) -> Result<T, std::io::Error>
{
	let nbi_key = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(NBI_REGISTRY_PATH)?;
	let value_result: std::io::Result<T> = nbi_key.get_value(value_name);
	return value_result;
}

pub fn set_nbi_value<T: ToRegValue>(value_name: &OsStr, value: &T) -> Option<std::io::Error>
{
	let base_key = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey_with_flags(NBI_REGISTRY_PATH, 2u32).ok()?;
	println!("{:?} | {}",value_name, value.to_reg_value().to_string());
	println!("{:?} | {:?}", value_name, value.to_reg_value());
	let value_result: Result<(), std::io::Error> = base_key.set_value(value_name, value);
	println!("{:?}", value_result);
	return match value_result
	{
		Ok(value) => { None }
		Err(err) => { Some(err) }
	}
}