use std::sync::Arc;
use std::{thread, time};

use super::program_data::{ModuleSpec, Status};
use super::{ControlMessage, ControlResponse, DB};
use crate::error::{Error, ErrorKind};

use crossbeam::crossbeam_channel::{Receiver, SendError, Sender};
use nix::unistd::{fork, ForkResult};

fn start_process(spec: ModuleSpec) -> Result<(ModuleSpec, Status), Error> {
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
            Ok((spec, Status::new(Some(child.into()), None)))
        }
        Ok(ForkResult::Child) => {
            println!("I'm a new child process");
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
    loop {
        let _ = db
            .retrieve(&module_name)
            .map_err(|err| Error::from(ErrorKind::DbRetrieve))
            .and_then(|spec| start_process(spec));
        let default_pause = time::Duration::from_secs(5);
        thread::sleep(default_pause);
    }
    // loop until module is removed
    // Get database entry for module.
    // fork, setup and exec process
    // save Started status & pid in database.
    // Wait on PID or channel update.
    // If child process ends, restart it.
    // If channel
}
