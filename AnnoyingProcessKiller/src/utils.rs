use windows::{
    Win32::{
        System::{
            Console::{
                AttachConsole, FreeConsole, ATTACH_PARENT_PROCESS
            },
            Threading::{
                TerminateProcess, OpenProcess, PROCESS_TERMINATE, PROCESS_QUERY_INFORMATION, OpenProcessToken
            }
        },
        Security::{
            TOKEN_ADJUST_PRIVILEGES, LookupPrivilegeValueA, TOKEN_PRIVILEGES,
            SE_PRIVILEGE_ENABLED, AdjustTokenPrivileges, TOKEN_PRIVILEGES_ATTRIBUTES
        },
        Foundation::{
            GetLastError, CloseHandle, HANDLE, LUID
        }
    },
    core::PCSTR
};

use std::ffi::CString;
use thiserror::Error;
use std::error::Error;


#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("Process termination failed -> {process} : {pid}) -> code: {errcode}")]
    TerminationFailed {
        process: String,
        pid: u32,
        errcode: u32
    },

    #[error("HANDLE is NULL -> {process} : {pid}) -> code: {errcode}")]
    NullHandle {
        process: String,
        pid: u32,
        errcode: u32
    },

    #[error("Failed to close HANDLE -> {process} : {pid}) -> code: {errcode}")]
    CloseHandleFailed {
        process: String,
        pid: u32,
        errcode: u32
    },

    #[error("Failed to open process token")]
    OpenProcessTokenFailed {
        errcode: u32
    },

    #[error("Failed to lookup privilege")]
    PrivilegeLookupFailed {
        name: String,
        errcode: u32
    },

    #[error("Adjust token privilege failed")]
    AdjustTokenPrivilegeFailed {
        name: String,
        errcode: u32
    }
}

pub fn set_privilege(name: &str, state: bool) -> Result<(), Box<dyn Error>> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION, false, std::process::id());
        if handle.is_invalid() {
            return Err(Box::new(ProcessError::NullHandle {
                process: env!("CARGO_PKG_NAME").to_string(),
                pid: std::process::id(),
                errcode: GetLastError().0
            }));
        }

        let mut token_handle = HANDLE(0);
        let res: bool = OpenProcessToken(
            handle,
            TOKEN_ADJUST_PRIVILEGES,
            &mut token_handle as *mut _
        ).into();

        if !res {
            return Err(Box::new(ProcessError::OpenProcessTokenFailed {
                errcode: GetLastError().0
            }));
        }

        let mut luid = LUID::default();
        let privilege = CString::new(name)?;
        let res: bool = LookupPrivilegeValueA(
            PCSTR::default(),
            PCSTR(privilege.as_ptr() as *const _),
            &mut luid as *mut _
        ).into();

        if !res {
            return Err(Box::new(ProcessError::PrivilegeLookupFailed {
                name: name.to_string(),
                errcode: GetLastError().0
            }));
        }

        let mut tp = TOKEN_PRIVILEGES::default();
        tp.PrivilegeCount = 1;
        tp.Privileges[0].Luid = luid;

        if state {
            tp.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;
        } else {
            tp.Privileges[0].Attributes = TOKEN_PRIVILEGES_ATTRIBUTES(0u32);
        }

        let res: bool = AdjustTokenPrivileges(
            token_handle,
            false,
            &tp as *const _,
            std::mem::size_of::<TOKEN_PRIVILEGES>() as u32,
            std::ptr::null_mut() as *mut _,
            std::ptr::null_mut() as *mut _
        ).into();

        if !res {
            return Err(Box::new(ProcessError::AdjustTokenPrivilegeFailed {
                name: name.to_string(),
                errcode: GetLastError().0
            }));
        }

        let res: bool = CloseHandle(handle).into();
        if !res {
            return Err(Box::new(ProcessError::CloseHandleFailed {
                process: env!("CARGO_PKG_NAME").to_string(),
                pid: std::process::id(),
                errcode: GetLastError().0
            }));
        }

        let res: bool = CloseHandle(token_handle).into();
        if !res {
            return Err(Box::new(ProcessError::CloseHandleFailed {
                process: env!("CARGO_PKG_NAME").to_string(),
                pid: std::process::id(),
                errcode: GetLastError().0
            }));
        }
    }

    Ok(())
}

pub fn hide_console() {
    unsafe {
        FreeConsole();
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

pub fn kill_process(name: &str, pid: u32) -> Result<(), ProcessError> {
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, false, pid);
        if handle.is_invalid() {
            return Err(ProcessError::NullHandle {
                process: name.to_string(),
                pid,
                errcode: GetLastError().0
            });
        }

        let res: bool = TerminateProcess(handle, 0).into();
        if !res {
            return Err(ProcessError::TerminationFailed {
                process: name.to_string(),
                pid,
                errcode: GetLastError().0
            });
        }

        let res: bool = CloseHandle(handle).into();
        if !res {
            return Err(ProcessError::CloseHandleFailed {
                process: name.to_string(),
                pid,
                errcode: GetLastError().0
            });
        }
    }

    Ok(())
}
