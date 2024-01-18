#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use std::fs::OpenOptions;
use std::io::Write;

use once_cell::sync::Lazy;
use retour::GenericDetour;
use windows::core::PCSTR;
use windows::Win32::Foundation::BOOL;
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::System::LibraryLoader::GetProcAddress;
use windows::Win32::System::LibraryLoader::LoadLibraryA;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows::Win32::System::SystemServices::DLL_PROCESS_DETACH;
use windows::Win32::UI::Input::KeyboardAndMouse::BlockInput;
use windows::Win32::UI::Input::KeyboardAndMouse::INPUT;

// This code is mostly a modified copy-paste of the MessageBoxA example from the
// retour-rs repo I won't pretend to deeply understand retour-rs or Windows API
// hooking in general If you have more experience with Rust (especially unsafe
// Rust) and have suggestions for improvement, please let me know

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

/// This assumes all was well and returns fBlockIt even if the call was not
/// successful. Probably should handle that, but meh.
#[allow(non_snake_case)]
extern "system" fn BlockInput_detour(fBlockIt: BOOL) -> BOOL
{
	unsafe { BlockInput_hook.disable().unwrap(); }
	if fBlockIt == BOOL(0)
	{
		unsafe { let _ = BlockInput(BOOL(0)); }
	};
	unsafe { BlockInput_hook.enable().unwrap();	}
	return fBlockIt;
}
#[allow(non_snake_case)]
extern "system" fn SendInput_detour(cInputs: u32, _: *const INPUT, _: i32) -> u32 { return cInputs; }

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: u32, _: *mut ()) -> bool
{
	if call_reason == DLL_PROCESS_ATTACH
	{
		let mut out_file = OpenOptions::new().append(true).open(r#"C:\Users\iamtheclaw\source\repos\Moo\Moo.NoBlockInput\test.txt"#).unwrap();
		let msg = "Started hook.\n";
		out_file.write_all(msg.as_bytes()).unwrap();
		out_file.flush().unwrap();
		unsafe
		{
			BlockInput_hook.disable().unwrap();
			SendInput_hook.disable().unwrap();
			let send_file = OpenOptions::new().read(true).open(r#"C:\Users\iamtheclaw\source\repos\Moo\Moo.NoBlockInput\send"#);
			if send_file.is_ok() { SendInput_hook.enable().unwrap(); }
			let block_file = OpenOptions::new().read(true).open(r#"C:\Users\iamtheclaw\source\repos\Moo\Moo.NoBlockInput\block"#);
			if block_file.is_ok() { BlockInput_hook.enable().unwrap(); }
		}
	}
	else if call_reason == DLL_PROCESS_DETACH
	{
		unsafe
		{
			BlockInput_hook.disable().unwrap();
			SendInput_hook.disable().unwrap();
		}
	}
	return true;
}
