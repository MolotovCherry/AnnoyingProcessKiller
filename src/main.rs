use std::collections::HashMap;
use std::error::Error;
use std::ffi::CString;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use sysinfo::{ProcessExt, System, SystemExt, RefreshKind, ProcessRefreshKind, PidExt};

use windows::Win32::System::Console::{AttachConsole, FreeConsole, ATTACH_PARENT_PROCESS};
use windows::Win32::System::SystemServices::SE_DEBUG_NAME;
use windows::Win32::System::Threading::{TerminateProcess, OpenProcess, PROCESS_TERMINATE, PROCESS_QUERY_INFORMATION, OpenProcessToken};
use windows::Win32::Foundation::{GetLastError, CloseHandle, HANDLE, LUID};
use windows::Win32::Security::{TOKEN_ADJUST_PRIVILEGES, LookupPrivilegeValueA, TOKEN_PRIVILEGES, SE_PRIVILEGE_ENABLED, AdjustTokenPrivileges, TOKEN_PRIVILEGES_ATTRIBUTES};
use windows::core::PCSTR;

use thiserror::Error;

use std::sync::mpsc::channel;
use ctrlc;

use std::thread;


lazy_static! {
    static ref DEFAULT: &'static str = r#"
{
    "processes": [
        "CompatTelRunner"
    ],
    "poll_frequency": 2
}
"#.trim_start();
}


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


#[derive(Serialize, Deserialize, Debug)]
struct Data {
    processes: Vec<String>,
    poll_frequency: u64
}


fn set_privilege(name: &str, state: bool) -> Result<(), Box<dyn Error>> {
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

fn hide_console() {
    unsafe {
        FreeConsole();
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

fn kill_process(name: &str, pid: u32) -> Result<(), ProcessError> {
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


fn main() -> Result<(), Box<dyn Error>> {
    // hide console
    let args: Vec<String> = std::env::args().collect();
    if let Some(v) = args.get(1) {
        if v == "-h" || v == "--help" {
            hide_console();
        }
    }

    // this privilege is required to kill SYSTEM processes
    // It requires Admin, but we enforce that in the manifest build.rs
    set_privilege(SE_DEBUG_NAME, true)?;

    let json = std::fs::read_to_string("config.json").unwrap_or_else(|_| {
        std::fs::write("config.json", DEFAULT.as_bytes()).expect("Failed to write file");
        DEFAULT.to_string()
    });

    let mut data: Data = serde_json::from_str(&json)?;

    let (tx, rx) = channel();

    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");

    // lowercase all of the entries
    for process in &mut data.processes.iter_mut() {
        *process = process.to_lowercase();
    }

    // all refresh kinds set to false
    let rk = RefreshKind::new();
    // everything set to false except process itself
    let prk = ProcessRefreshKind::new();
    let mut sys = System::new_with_specifics(rk.with_processes(prk));

    thread::spawn(move || {
        loop {
            sys.refresh_processes_specifics(prk);

            let list: HashMap<_, _> = sys.processes().iter().map(|(pid, process)| (process.name().to_lowercase(), pid.as_u32())).collect();
            for _p in &data.processes {
                let process = &*format!("{_p}.exe");
                if list.contains_key(process) {
                    let pid = *list.get(process).unwrap();
                    println!("Killing process {process} : {pid}");
                    kill_process(process, pid)?;
                }
            }

            std::thread::sleep(std::time::Duration::from_secs(data.poll_frequency));
        }

        #[allow(unreachable_code)]
        Ok::<(), ProcessError>(())
    });


    rx.recv()?;

    Ok(())
}
