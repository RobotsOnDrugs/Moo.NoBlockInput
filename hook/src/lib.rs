#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use std::io::Result;

use once_cell::sync::Lazy;

use retour::GenericDetour;

use windows::core::PCSTR;
#[cfg(debug_assertions)]
use windows::Win32::System::Console::{AllocConsole, FreeConsole};
use windows::Win32::Foundation::BOOL;
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::System::LibraryLoader::GetProcAddress;
use windows::Win32::System::LibraryLoader::LoadLibraryA;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows::Win32::System::SystemServices::DLL_PROCESS_DETACH;
use windows::Win32::UI::Input::KeyboardAndMouse::BlockInput;

use windows_sys::Win32::UI::Input::KeyboardAndMouse::INPUT;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::SendInput;

use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;

const NBI_REGISTRY_PATH: &str = r#"SOFTWARE\Moo\NoBlockInput"#;
const BLOCKINPUT_HOOK_ENABLE_NAME: &str = "BlockInputHookEnabled";
const SENDINPUT_HOOK_ENABLE_NAME: &str = "SendInputHookEnabled";

#[allow(non_camel_case_types)]
type BlockInput_signature = extern "system" fn(BOOL) -> BOOL;
#[allow(non_camel_case_types)]
type SendInput_signature = extern "system" fn(u32, *const INPUT, i32) -> u32;

#[allow(non_upper_case_globals)]
static BlockInput_hook: Lazy<GenericDetour<BlockInput_signature>> =
	Lazy::new(||
	{
		let library_handle = unsafe { LoadLibraryA(PCSTR(b"user32.dll\0".as_ptr() as _)) }.unwrap();
		let address = unsafe { GetProcAddress(library_handle, PCSTR(b"BlockInput\0".as_ptr() as _)) };
		let ori: BlockInput_signature = unsafe { std::mem::transmute(address) };
		return unsafe { GenericDetour::new(ori, BlockInput_detour).unwrap() };
	});
#[allow(non_upper_case_globals)]
static SendInput_hook: Lazy<GenericDetour<SendInput_signature>> =
	Lazy::new(||
	{
		let library_handle = unsafe { LoadLibraryA(PCSTR(b"user32.dll\0".as_ptr() as _)) }.unwrap();
		let address = unsafe { GetProcAddress(library_handle, PCSTR(b"SendInput\0".as_ptr() as _)) };
		let ori: SendInput_signature = unsafe { std::mem::transmute(address) };
		return unsafe { GenericDetour::new(ori, SendInput_detour).unwrap() };
	});

/// This assumes all was well and returns fBlockIt even if the call was not successful.
/// Probably should handle that, but meh.
/// The default is to enable the hook when there is an error.
#[allow(non_snake_case)]
extern "system" fn BlockInput_detour(fBlockIt: BOOL) -> BOOL
{
	unsafe
	{
		BlockInput_hook.disable().unwrap();
		match is_hook_enabled(BLOCKINPUT_HOOK_ENABLE_NAME)
		{
			// If the hook is enabled, still pass through calls to unblock
			true => { if fBlockIt == BOOL(0) { let _ = BlockInput(BOOL(0)); }; }
			false => { let _ = BlockInput(fBlockIt); }
		}
		BlockInput_hook.enable().unwrap();
	}
	return fBlockIt;
}
#[allow(non_snake_case)]
extern "system" fn SendInput_detour(cInputs: u32, pInputs: *const INPUT, cbSize: i32) -> u32
{
	if is_hook_enabled(SENDINPUT_HOOK_ENABLE_NAME) { return cInputs; }
	unsafe
	{
		SendInput_hook.disable().unwrap();
		let inputs_processed = SendInput(cInputs, pInputs, cbSize);
		SendInput_hook.enable().unwrap();
		return inputs_processed;
	}
}

fn is_hook_enabled(reg_value_name: &str) -> bool
{
	let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
	let nbi_registry = hklm.open_subkey(NBI_REGISTRY_PATH);
	return match nbi_registry
	{
		Ok(key) =>
		{
			let enable_value: Result<u32> = key.get_value(reg_value_name);
			match enable_value
			{
				Ok(val) =>
				{
					match val
					{
						0u32 => false,
						_ => true
					}
				}
				Err(_) => { true }
			}
		}
		Err(_) => { true }
	};
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: u32, _: *mut ()) -> bool
{
	if call_reason == DLL_PROCESS_ATTACH
	{
		unsafe
		{
			SendInput_hook.enable().unwrap();
			BlockInput_hook.enable().unwrap();
			#[cfg(debug_assertions)]
			{
				let console = AllocConsole();
				match console
				{
					Ok(_) => { println!("Allocating console was successful."); }
					// I've played with AllocConsole in .NET and gotten a console creation despite an error, so it could still work.
					// I won't bother with GetLastError (and potentially writing to a file if there is no console) unless I start seeing errors when used in this program.
					Err(_) => { eprintln!("Error allocating console."); }
				};
			}
		}
	}
	else if call_reason == DLL_PROCESS_DETACH
	{
		unsafe
		{
			BlockInput_hook.disable().unwrap();
			SendInput_hook.disable().unwrap();
			#[cfg(debug_assertions)]
			FreeConsole().unwrap();
		}
	}
	return true;
}
