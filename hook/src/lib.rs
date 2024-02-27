#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]


use anyhow::Result;

use log::info;
use log::trace;
use log::warn;

use once_cell::sync::Lazy;

use retour::GenericDetour;

use simplelog::CombinedLogger;

use windows::core::PCSTR;
use windows::Win32::Foundation::BOOL;
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::System::LibraryLoader::GetProcAddress;
use windows::Win32::System::LibraryLoader::LoadLibraryA;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows::Win32::System::SystemServices::DLL_PROCESS_DETACH;
use windows::Win32::UI::Input::KeyboardAndMouse::BlockInput;

use windows_sys::Win32::UI::Input::KeyboardAndMouse::INPUT;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::mouse_event;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::MOUSE_EVENT_FLAGS;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::SendInput;

use noblock_input_common::configuration::InjectorConfig;
use noblock_input_common::logging::create_logger;
use noblock_input_common::registry::get_nbi_value;

const BLOCKINPUT_HOOK_ENABLE_NAME: &str = "BlockInputHookEnabled";
const SENDINPUT_HOOK_ENABLE_NAME: &str = "SendInputHookEnabled";

#[allow(non_camel_case_types)]
type BlockInput_signature = extern "system" fn(BOOL) -> BOOL;
#[allow(non_camel_case_types)]
type SendInput_signature = extern "system" fn(u32, *const INPUT, i32) -> u32;
#[allow(non_camel_case_types)]
type mouse_event_signature = extern "system" fn(MOUSE_EVENT_FLAGS, i32, i32, i32, usize);

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
#[allow(non_upper_case_globals)]
static mouse_event_hook: Lazy<GenericDetour<mouse_event_signature>> =
	Lazy::new(||
		{
			let library_handle = unsafe { LoadLibraryA(PCSTR(b"user32.dll\0".as_ptr() as _)) }.unwrap();
			let address = unsafe { GetProcAddress(library_handle, PCSTR(b"mouse_event\0".as_ptr() as _)) };
			let ori: mouse_event_signature = unsafe { std::mem::transmute(address) };
			return unsafe { GenericDetour::new(ori, mouse_event_detour).unwrap() };
		});

/// This assumes all was well and returns fBlockIt even if the call was not successful.
/// Probably should handle that, but meh.
/// The default is to enable the hook when there is an error.
#[allow(non_snake_case)]
extern "system" fn BlockInput_detour(fBlockIt: BOOL) -> BOOL
{
	trace!("BlockInput detour was reached.");
	unsafe
	{
		BlockInput_hook.disable().unwrap();
		match is_hook_enabled(BLOCKINPUT_HOOK_ENABLE_NAME)
		{
			// If the hook is enabled, still pass through calls to unblock
			true =>
			{
				trace!("BlockInput was blocked.");
				if fBlockIt == BOOL(0) { let _ = BlockInput(BOOL(0)); };
			}
			false =>
			{
				trace!("BlockInput was passed through.");
				let _ = BlockInput(fBlockIt);
			}
		}
		BlockInput_hook.enable().unwrap();
	}
	return fBlockIt;
}
#[allow(non_snake_case)]
extern "system" fn SendInput_detour(cInputs: u32, pInputs: *const INPUT, cbSize: i32) -> u32
{
	trace!("SendInput detour was reached.");
	if is_hook_enabled(SENDINPUT_HOOK_ENABLE_NAME)
	{
		trace!("SendInput was blocked.");
		return cInputs;
	}
	unsafe
	{
		SendInput_hook.disable().unwrap();
		let inputs_processed = SendInput(cInputs, pInputs, cbSize);
		SendInput_hook.enable().unwrap();
		trace!("SendInput was passed through.");
		return inputs_processed;
	}
}
#[allow(non_snake_case)]
extern "system" fn mouse_event_detour(dwFlags: MOUSE_EVENT_FLAGS, dx: i32, dy: i32, dwData: i32, dwExtraInfo: usize)
{
	if is_hook_enabled(SENDINPUT_HOOK_ENABLE_NAME) { return; }
	unsafe { mouse_event(dwFlags, dx, dy, dwData, dwExtraInfo); }
}

fn is_hook_enabled(reg_value_name: &str) -> bool
{
	let value: Result<u32, std::io::Error> = get_nbi_value(reg_value_name);
	return match value
	{
		Ok(value) => { !matches!(value, 0u32) }
		Err(err) => { warn!("{err}"); true }
	}
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: u32, _: *mut ()) -> bool
{
	if call_reason == DLL_PROCESS_ATTACH
	{
		let configuration = InjectorConfig::try_new(Some(true), None);
		if configuration.is_ok()
		{
			let logger = create_logger(std::env::current_exe().unwrap(), configuration.unwrap().log_directory);
			CombinedLogger::init(logger.loggers).unwrap();
		};
		unsafe
		{
			SendInput_hook.enable().unwrap();
			BlockInput_hook.enable().unwrap();
			mouse_event_hook.enable().unwrap();
		}
		info!("DLL was successfully injected.");
	}
	else if call_reason == DLL_PROCESS_DETACH
	{
		unsafe
		{
			BlockInput_hook.disable().unwrap();
			SendInput_hook.disable().unwrap();
			mouse_event_hook.disable().unwrap();
		}
	}
	return true;
}