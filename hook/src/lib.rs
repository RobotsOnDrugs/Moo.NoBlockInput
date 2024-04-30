#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]


use std::ffi::c_void;

use anyhow::Result;
use log::debug;
use log::info;
use log::trace;
use log::warn;
use once_cell::sync::Lazy;
use retour::Function;
use retour::GenericDetour;
use retour::HookableWith;
use simplelog::CombinedLogger;
use windows::core::PCSTR;
use windows::Win32::Foundation::BOOL;
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::System::LibraryLoader::GetProcAddress;
use windows::Win32::System::LibraryLoader::LoadLibraryA;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows::Win32::System::SystemServices::DLL_PROCESS_DETACH;
use windows::Win32::UI::Input::KeyboardAndMouse::BlockInput;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::Graphics::Gdi::CDS_TYPE;
use windows_sys::Win32::Graphics::Gdi::ChangeDisplaySettingsA;
use windows_sys::Win32::Graphics::Gdi::ChangeDisplaySettingsExA;
use windows_sys::Win32::Graphics::Gdi::ChangeDisplaySettingsExW;
use windows_sys::Win32::Graphics::Gdi::ChangeDisplaySettingsW;
use windows_sys::Win32::Graphics::Gdi::DEVMODEA;
use windows_sys::Win32::Graphics::Gdi::DEVMODEW;
use windows_sys::Win32::Graphics::Gdi::DISP_CHANGE;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::INPUT;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::mouse_event;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::MOUSE_EVENT_FLAGS;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::SendInput;

use noblock_input_common::configuration::InjectorConfig;
use noblock_input_common::logging::create_logger;
use noblock_input_common::registry::get_nbi_value;

const BLOCKINPUT_HOOK_ENABLE_NAME: &str = "BlockInputHookEnabled";
const SENDINPUT_HOOK_ENABLE_NAME: &str = "SendInputHookEnabled";
// const CHANGE_DISPLAY_SETTINGS_HOOK_ENABLE_NAME: &str = "ChangeDisplaySettingsHookEnabled";

type BlockInput_signature = unsafe extern "system" fn(BOOL) -> BOOL;
type SendInput_signature = unsafe extern "system" fn(u32, *const INPUT, i32) -> u32;
type mouse_event_signature = unsafe extern "system" fn(MOUSE_EVENT_FLAGS, i32, i32, i32, usize);
type ChangeDisplaySettingsA_signature = unsafe extern "system" fn(*const DEVMODEA, CDS_TYPE) -> DISP_CHANGE;
type ChangeDisplaySettingsW_signature = unsafe extern "system" fn(*const DEVMODEW, CDS_TYPE) -> DISP_CHANGE;
type ChangeDisplaySettingsExA_signature = unsafe extern "system" fn(windows_sys::core::PCSTR, *const DEVMODEA, HWND, CDS_TYPE, *const c_void) -> DISP_CHANGE;
type ChangeDisplaySettingsExW_signature = unsafe extern "system" fn(windows_sys::core::PCWSTR, *const DEVMODEW, HWND, CDS_TYPE, *const c_void) -> DISP_CHANGE;

unsafe fn make_lazy<T: HookableWith<D> + Sized, D: Function>(dll_name: &str, fn_name: &str, detour: D) -> GenericDetour<T>
{
	let dll_name = format!("{dll_name}\0");
	let dll_name = PCSTR::from_raw(dll_name.as_bytes().as_ptr());
	let fn_name = format!("{fn_name}\0");
	let fn_name = PCSTR::from_raw(fn_name.as_bytes().as_ptr());
	
	let library_handle = LoadLibraryA(dll_name).unwrap();
	let address = GetProcAddress(library_handle, fn_name);
	let ori: T = std::mem::transmute_copy(&address.unwrap());
	return GenericDetour::new(ori, detour).unwrap();
}

static BlockInput_hook: Lazy<GenericDetour<BlockInput_signature>> =
	unsafe { Lazy::new(|| return make_lazy::<BlockInput_signature, BlockInput_signature>("user32.dll", "BlockInput", BlockInput_detour)) };
static SendInput_hook: Lazy<GenericDetour<SendInput_signature>> =
	unsafe { Lazy::new(|| return make_lazy::<SendInput_signature, SendInput_signature>("user32.dll", "SendInput", SendInput_detour)) };
static mouse_event_hook: Lazy<GenericDetour<mouse_event_signature>> =
	unsafe { Lazy::new(|| return make_lazy::<mouse_event_signature, mouse_event_signature>("user32.dll", "mouse_event", mouse_event_detour)) };
static ChangeDisplaySettingsA_hook: Lazy<GenericDetour<ChangeDisplaySettingsA_signature>> =
	unsafe { Lazy::new(|| return make_lazy::<ChangeDisplaySettingsA_signature, ChangeDisplaySettingsA_signature>("user32.dll", "ChangeDisplaySettingsA", ChangeDisplaySettingsA_detour)) };
static ChangeDisplaySettingsW_hook: Lazy<GenericDetour<ChangeDisplaySettingsW_signature>> =
	unsafe { Lazy::new(|| return make_lazy::<ChangeDisplaySettingsW_signature, ChangeDisplaySettingsW_signature>("user32.dll", "ChangeDisplaySettingsW", ChangeDisplaySettingsW_detour)) };
static ChangeDisplaySettingsExA_hook: Lazy<GenericDetour<ChangeDisplaySettingsExA_signature>> =
	unsafe { Lazy::new(|| return make_lazy::<ChangeDisplaySettingsExA_signature, ChangeDisplaySettingsExA_signature>("user32.dll", "ChangeDisplaySettingsExA", ChangeDisplaySettingsExA_detour)) };
static ChangeDisplaySettingsExW_hook: Lazy<GenericDetour<ChangeDisplaySettingsExW_signature>> =
	unsafe { Lazy::new(|| return make_lazy::<ChangeDisplaySettingsExW_signature, ChangeDisplaySettingsExW_signature>("user32.dll", "ChangeDisplaySettingsExW", ChangeDisplaySettingsExW_detour)) };

/// This assumes all was well and returns fBlockIt even if the call was not
/// successful. Probably should handle that, but meh.
/// The default is to enable the hook when there is an error.
unsafe extern "system" fn BlockInput_detour(fBlockIt: BOOL) -> BOOL
{
	trace!("BlockInput detour was reached.");
	BlockInput_hook.disable().unwrap();
	match is_hook_enabled(BLOCKINPUT_HOOK_ENABLE_NAME)
	{
		// If the hook is enabled, still pass through calls to unblock
		true =>
		{
			trace!("BlockInput was blocked.");
			if fBlockIt == BOOL(0) { let _ = BlockInput(BOOL(0)); };
		},
		false =>
		{
			trace!("BlockInput was passed through.");
			let _ = BlockInput(fBlockIt);
		}
	}
	BlockInput_hook.enable().unwrap();
	return fBlockIt;
}
unsafe extern "system" fn SendInput_detour(cInputs: u32, pInputs: *const INPUT, cbSize: i32) -> u32
{
	trace!("SendInput detour was reached.");
	if is_hook_enabled(SENDINPUT_HOOK_ENABLE_NAME)
	{
		trace!("SendInput was blocked.");
		return cInputs;
	}
	SendInput_hook.disable().unwrap();
	let inputs_processed = SendInput(cInputs, pInputs, cbSize);
	SendInput_hook.enable().unwrap();
	trace!("SendInput was passed through.");
	return inputs_processed;
}
unsafe extern "system" fn mouse_event_detour(dwFlags: MOUSE_EVENT_FLAGS, dx: i32, dy: i32, dwData: i32, dwExtraInfo: usize)
{
	if is_hook_enabled(SENDINPUT_HOOK_ENABLE_NAME) { return; }
	mouse_event_hook.disable().unwrap();
	mouse_event(dwFlags, dx, dy, dwData, dwExtraInfo);
	mouse_event_hook.enable().unwrap();
}

unsafe extern "system" fn ChangeDisplaySettingsA_detour(lpdevmode: *const DEVMODEA, dwflags: CDS_TYPE) -> DISP_CHANGE
{
	debug!("ChangeDisplaySettingsA: device mode: {:?} | CDS type: {:?}", lpdevmode, dwflags);
	// if is_hook_enabled(CHANGE_DISPLAY_SETTINGS_HOOK_ENABLE_NAME) { } // return DISP_CHANGE(0);
	ChangeDisplaySettingsA_hook.disable().unwrap();
	let disp_change = ChangeDisplaySettingsA(lpdevmode, dwflags);
	ChangeDisplaySettingsA_hook.enable().unwrap();
	return disp_change;
}
unsafe extern "system" fn ChangeDisplaySettingsW_detour(lpdevmode: *const DEVMODEW, dwflags: CDS_TYPE) -> DISP_CHANGE
{
	debug!("ChangeDisplaySettingsW: device mode: {:?} | CDS type: {:?}", lpdevmode, dwflags);
	// if is_hook_enabled(CHANGE_DISPLAY_SETTINGS_HOOK_ENABLE_NAME) { } // return DISP_CHANGE(0);
	ChangeDisplaySettingsW_hook.disable().unwrap();
	let disp_change = ChangeDisplaySettingsW(lpdevmode, dwflags);
	ChangeDisplaySettingsW_hook.enable().unwrap();
	return disp_change;
}
unsafe extern "system" fn ChangeDisplaySettingsExA_detour(lpszdevicename: windows_sys::core::PCSTR, lpdevmode: *const DEVMODEA, hwnd: HWND, dwflags: CDS_TYPE, lparam: *const c_void) -> DISP_CHANGE
{
	info!("ChangeDisplaySettingsExA: device name: {:?} | device mode: {:?} | hwnd: {:?} | CDS type: {:?} | lparam: {:?}", lpszdevicename, lpdevmode, hwnd, dwflags, lparam);
	// if is_hook_enabled(CHANGE_DISPLAY_SETTINGS_HOOK_ENABLE_NAME) { } // return DISP_CHANGE(0);
	ChangeDisplaySettingsExA_hook.disable().unwrap();
	let disp_change = ChangeDisplaySettingsExA(lpszdevicename, lpdevmode, hwnd, dwflags, lparam);
	ChangeDisplaySettingsExA_hook.enable().unwrap();
	return disp_change;
}
unsafe extern "system" fn ChangeDisplaySettingsExW_detour(lpszdevicename: windows_sys::core::PCWSTR, lpdevmode: *const DEVMODEW, hwnd: HWND, dwflags: CDS_TYPE, lparam: *const c_void) -> DISP_CHANGE
{
	info!("ChangeDisplaySettingsExA: device name: {:?} | device mode: {:?} | hwnd: {:?} | CDS type: {:?} | lparam: {:?}", lpszdevicename, lpdevmode, hwnd, dwflags, lparam);
	// if is_hook_enabled(CHANGE_DISPLAY_SETTINGS_HOOK_ENABLE_NAME) { } // return DISP_CHANGE(0);
	ChangeDisplaySettingsExW_hook.disable().unwrap();
	let disp_change = ChangeDisplaySettingsExW(lpszdevicename, lpdevmode, hwnd, dwflags, lparam);
	ChangeDisplaySettingsExW_hook.enable().unwrap();
	return disp_change;
}

fn is_hook_enabled(reg_value_name: &str) -> bool
{
	let value: Result<u32, std::io::Error> = get_nbi_value(reg_value_name);
	return match value
	{
		Ok(value) => !matches!(value, 0u32),
		Err(err) => { warn!("{err}"); true }
	};
}

// .unwrap() calls here can't be handled, so there's nothing to do but panic.
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
unsafe extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: u32, _: *mut ()) -> bool
{
	if call_reason == DLL_PROCESS_ATTACH
	{
		let configuration = InjectorConfig::get_configuration();
		if configuration.is_ok()
		{
			let configuration = configuration.unwrap();
			let logger = create_logger(std::env::current_exe(), configuration.log_directory, configuration.log_level);
			let _ = CombinedLogger::init(logger.loggers);
		};
		BlockInput_hook.enable().unwrap();
		SendInput_hook.enable().unwrap();
		mouse_event_hook.enable().unwrap();
		ChangeDisplaySettingsA_hook.enable().unwrap();
		ChangeDisplaySettingsW_hook.enable().unwrap();
		ChangeDisplaySettingsExA_hook.enable().unwrap();
		ChangeDisplaySettingsExW_hook.enable().unwrap();
		info!("DLL was successfully injected.");
	}
	else if call_reason == DLL_PROCESS_DETACH
	{
		BlockInput_hook.disable().unwrap();
		SendInput_hook.disable().unwrap();
		mouse_event_hook.disable().unwrap();
		ChangeDisplaySettingsA_hook.disable().unwrap();
		ChangeDisplaySettingsW_hook.disable().unwrap();
		ChangeDisplaySettingsExA_hook.disable().unwrap();
		ChangeDisplaySettingsExW_hook.disable().unwrap();
	}
	return true;
}