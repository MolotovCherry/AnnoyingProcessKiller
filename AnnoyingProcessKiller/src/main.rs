mod utils;

use WMI_Query;
use tokio::select;
use WMI_Query::Win32_Process::Win32_Process;

use core::convert::From;
use std::{error::Error};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use windows::Win32::System::SystemServices::SE_DEBUG_NAME;

use ctrlc;


lazy_static! {
    static ref DEFAULT: &'static str = r#"
{
    "processes": [
        "CompatTelRunner.exe"
    ]
}
"#.trim_start();
}


#[derive(Serialize, Deserialize, Debug)]
struct Data {
    processes: Vec<String>
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // hide console
    let args: Vec<String> = std::env::args().collect();
    if let Some(v) = args.get(1) {
        if v == "-h" || v == "--help" {
            utils::hide_console();
        }
    }

    // this privilege is required to kill SYSTEM processes
    // It requires Admin, but we enforce that in the manifest build.rs
    utils::set_privilege(SE_DEBUG_NAME, true)?;

    let json = std::fs::read_to_string("config.json").unwrap_or_else(|_| {
        std::fs::write("config.json", DEFAULT.as_bytes()).expect("Failed to write file");
        DEFAULT.to_string()
    });

    let mut data: Data = serde_json::from_str(&json)?;

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);

    ctrlc::set_handler(move || tx.try_send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");

    // lowercase all of the entries
    for process in &mut data.processes.iter_mut() {
        *process = process.to_lowercase();
    }

    let wmi_con = WMI_Query::WMIConnection::new()?;
    let rx2 = wmi_con.exec_notification_query_async(
        "SELECT * FROM __InstanceCreationEvent WITHIN 2 WHERE TargetInstance ISA 'Win32_Process'"
    )?;

    loop {
        select! {
            // ctrl c break
            _ = rx.recv() => break,

            Ok(Ok(process)) = rx2.recv() => {
                //println!("got process {:?}", process);

                //println!("all properties: {:?}", process.get_property_names());
                //println!("property: {:?}", process.get_property("TargetInstance"));
                let inst = process.get_embedded_object("TargetInstance")?;
                //println!("embedded {:?}", inst);
                //println!("embedded props {:?}", inst.get_property_names());
                //println!("process name: {:?}", inst.get_property("Name"));
                //println!("properties! {:?}", inst.get_properties(false));
                //println!("");

                let res = Win32_Process::from(inst);
                println!("Started {}, {}", res.Name, res.ProcessId);
                if data.processes.contains(&res.Name.to_lowercase()) {
                    println!("{} is disallowed! Killed!", res.Name);
                    utils::kill_process(&res.Name, res.ProcessId as u32)?;
                } else {
                    println!("{} is allowed", res.Name);
                }
                println!();
            }
        }
    }

    Ok(())
}
