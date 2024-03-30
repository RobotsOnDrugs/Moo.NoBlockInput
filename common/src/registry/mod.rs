#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]


use anyhow::Result;

use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;
use winreg::types::FromRegValue;

const NBI_REGISTRY_PATH: &str = r#"SOFTWARE\Moo\NoBlockInput"#;

pub fn get_nbi_value<T: FromRegValue>(value_name: &str) -> Result<T, std::io::Error>
{
	let nbi_key = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(NBI_REGISTRY_PATH)?;
	let value_result: std::io::Result<T> = nbi_key.get_value(value_name);
	return value_result;
}