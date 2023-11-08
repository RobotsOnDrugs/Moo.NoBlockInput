use once_cell::sync::Lazy;

use retour::GenericDetour;

use windows::core::PCSTR;
use windows::Win32::Foundation::{BOOL, HINSTANCE};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};
use windows::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_THREAD_ATTACH};
use windows::Win32::UI::Input::KeyboardAndMouse::BlockInput;

// This code is mostly a modified copy-paste of the MessageBoxA example from the retour-rs repo
// I won't pretend to deeply understand retour-rs or Windows API hooking in general
// If you have more experience with Rust (especially unsafe Rust) and have suggestions for improvement, please let me know

#[allow(non_camel_case_types)]
type BlockInput_signature = extern "system" fn(BOOL) -> BOOL;

#[allow(non_upper_case_globals)]
static BlockInput_hook: Lazy<GenericDetour<BlockInput_signature>> =
    Lazy::new(||
        {
            let library_handle = unsafe { LoadLibraryA(PCSTR(b"user32.dll\0".as_ptr() as _)) }.unwrap();
            let address = unsafe { GetProcAddress(library_handle, PCSTR(b"BlockInput\0".as_ptr() as _)) };
            let ori: BlockInput_signature = unsafe { std::mem::transmute(address) };
            return unsafe { GenericDetour::new(ori, BlockInput_detour).unwrap() };
        });

#[allow(non_snake_case)]
extern "system" fn BlockInput_detour(fBlockIt: BOOL) -> BOOL
{
    unsafe { BlockInput_hook.disable().unwrap(); }
    if fBlockIt == BOOL(0)
    {
        unsafe { let _ = BlockInput(BOOL(0)); }
    };
    unsafe { BlockInput_hook.enable().unwrap(); }
    return fBlockIt;
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: u32,  _: *mut ()) -> bool
{
    match call_reason
    {
        DLL_PROCESS_ATTACH | DLL_THREAD_ATTACH => { unsafe { BlockInput_hook.enable().unwrap(); } },
        _ => ()
    }
    return true;
}

