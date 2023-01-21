use lazy_static::lazy_static;
use std::{
    env,
    fs::{read_to_string, remove_file, File},
    io::Write,
    path::Path,
    process::{exit, Command},
};
use sysinfo::{Pid, ProcessExt, ProcessRefreshKind, RefreshKind, System, SystemExt};

use crate::structs::{KillParse, KillService};

const KILLED_FILENAME: &str = "killed.list";

lazy_static! {
    static ref KILLED_PATH: &'static Path = Path::new(KILLED_FILENAME);
    static ref DRY_RUN: bool = env::args().any(|arg| &arg == "--dry-run");
}

struct ExtractedProcess {
    name: String,
    memory: u64,
    pid: Pid,
    exe: String,
}

pub fn restore() -> Result<String, String> {
    let killed = match read_to_string(*KILLED_PATH) {
        Ok(x) => x,
        Err(err) => return Err(format!("{err}")),
    };
    let mut succeded = 0;

    if *DRY_RUN {
        succeded = killed.lines().count();
    } else {
        for line in killed.lines() {
            let result: bool = if *DRY_RUN {
                true
            } else if line.starts_with("||") {
                restore_service(line.get(2..).unwrap())
            } else {
                restore_process(line, false)
            };
            if result {
                succeded += 1
            };
        }
    };

    if let Err(error) = remove_file(*KILLED_PATH) {
        eprintln!("Error deleting file: {error}")
    }

    Ok(format!("Restored {succeded} processes"))
}

pub fn restore_process(process: &str, admin: bool) -> bool {
    match Command::new("cmd")
        .arg("/C")
        .arg("start")
        .arg("")
        .args(process.split('|'))
        .env(
            "__COMPAT_LAYER",
            if admin {
                "RunAsHighest"
            } else {
                "RUNASINVOKER"
            },
        )
        .spawn()
    {
        Ok(_) => return true,
        Err(err) => eprintln!("{err}"),
    };
    false
}

fn restore_service(name: &str) -> bool {
    match Command::new("net").arg("start").arg(name).status() {
        Ok(status) => {
            if !status.success() {
                eprintln!("Failed to start {name}");
            } else {
                return true;
            }
        }
        Err(err) => eprintln!("Error starting service: {err}"),
    };
    false
}

pub fn kill(killable: &KillParse) -> Result<String, String> {
    let process = ProcessRefreshKind::new();
    let kind = RefreshKind::new().with_processes(process);

    let sys = System::new_with_specifics(kind);

    let mut processes = get_data(&sys);
    processes.sort_by_key(|p| p.memory);

    let mut saved_mem = 0;
    let mut killed = vec![];

    for process in processes {
        let killable = match killable
            .processes
            .iter()
            .find(|k| k.name == process.name.as_str())
        {
            None => continue,
            Some(j) => j,
        };

        let result = if *DRY_RUN {
            true
        } else {
            kill_process(&sys, &process)
        };

        if result {
            saved_mem += process.memory;
            if killable.restore {
                killed.push(process.exe);
            }
        } else {
            eprintln!("Failed to kill {}", killable.name)
        }
    }

    killable.services.iter().for_each(|kill| {
        let result = if *DRY_RUN { true } else { kill_service(kill) };

        if result {
            if kill.restore {
                killed.push("||".to_owned() + kill.name.as_str());
            }
        } else {
            eprintln!("Failed to kill {}", kill.name);
        }
    });

    let mut file = match File::create(*KILLED_PATH) {
        Ok(file) => file,
        Err(error) => {
            println!("Error creating file: {error}");
            exit(1)
        }
    };
    if let Err(error) = file.write_all(killed.join("\n").as_bytes()) {
        eprintln!("Error writing to file: {error}");
    }

    Ok(format!("Saved {saved_mem} MB"))
}

fn kill_service(killable: &KillService) -> bool {
    match Command::new("net").arg("stop").arg(&killable.name).status() {
        Ok(status) => status.success(),
        Err(err) => {
            eprintln!("{err}");
            false
        }
    }
}

fn kill_process(sys: &System, process: &ExtractedProcess) -> bool {
    sys.process(process.pid).unwrap().kill()
}

fn get_data(sys: &System) -> Vec<ExtractedProcess> {
    sys.processes()
        .iter()
        .map(|(pid, process)| ExtractedProcess {
            name: process.name().to_owned(),
            memory: process.memory() / 1_100_100,
            pid: *pid,
            exe: process.cmd().join("|"),
        })
        .collect()
}
