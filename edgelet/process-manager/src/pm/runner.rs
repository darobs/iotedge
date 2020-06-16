use std::sync::Arc;
use std::{thread, time};
//use tokio::process::Command;
use super::program_data::{ModuleSpec, Status};
use super::{ControlMessage, ControlResponse, DB};
use crate::error::{Error, ErrorKind};

use crossbeam::crossbeam_channel::{Receiver, SendError, Sender};
use nix::unistd::{fork, ForkResult};

fn start_process(spec: ModuleSpec) -> Result<ModuleSpec, Error> {
    // let outputs = File::create("out.txt")?;
    // let errors = outputs.try_clone()?;// Command::new("ls")
    //     .args(&[".", "oops"])
    //     .stdout(Stdio::from(outputs))
    //     .stderr(Stdio::from(errors))
    match fork() {
        Ok(ForkResult::Parent { child, .. }) => {
            println!(
                "Continuing execution in parent process, new child has pid: {}",
                child
            );
            Ok(spec.with_status(Status::new(Some(child.as_raw() as u32), None)))
        }
        Ok(ForkResult::Child) => {
            println!("I'm a new child process");
            std::process::exit(0_i32);
            Err(Error::from(ErrorKind::ForkFailed))
        }
        Err(_) => Err(Error::from(ErrorKind::ForkFailed)),
    }
}

pub fn control_loop(
    module_name: String,
    db: Arc<DB>,
    control_rx: Receiver<ControlMessage>,
    response: Sender<ControlResponse>,
) {
    let mut loop_count = 0;
    loop {
        let spec = db
            .retrieve(&module_name)
            .map_err(|err| Error::from(ErrorKind::DbRetrieve));
        if let Ok(spec) = spec {
            let spec = start_process(spec).and_then(|spec| {
                let mut lock = db.lock().map_err(|_| Error::from(ErrorKind::DbLock))?;
                lock.insert(&module_name, spec)
                    .map_err(|_| Error::from(ErrorKind::DbInsert))
            });
            if let Ok(_) = spec {
                db.flush().map_err(|_| Error::from(ErrorKind::DbFlush));
            }
        } else {
            println!("Failed to retrieve module");
        }

        loop_count += 1;
        if loop_count > 10 {
            break;
        }
        let default_pause = time::Duration::from_secs(5);
        thread::sleep(default_pause);
    }
    response.send(ControlResponse::Stopped);

    // loop until module is removed
    // Get database entry for module.
    // fork, setup and exec process
    // save Started status & pid in database.
    // Wait on PID or channel update.
    // If child process ends, restart it.
    // If channel
}
