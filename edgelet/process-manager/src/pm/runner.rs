use std::fs::File;
use std::sync::Arc;

use super::program_data::{ModuleSpec, Status};
use super::{ControlMessage, ControlResponse, DB};
use crate::error::{Error, ErrorKind};

use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tokio::time;
use users::{get_group_by_name, get_user_by_name};

fn lookup_uid(uid: Option<&str>) -> Option<u32> {
    // could be a non-zero integer
    uid.and_then(|u_str| {
        u_str
            .parse::<u32>()
            .ok()
            .map(|uid_t| Some(uid_t))
            .unwrap_or_else(|| get_user_by_name(u_str).map(|user| user.uid()))
    })
}

fn lookup_gid(gid: Option<&str>) -> Option<u32> {
    // could be a non-zero integer
    gid.and_then(|g_str| {
        g_str
            .parse::<u32>()
            .ok()
            .map(|gid_t| Some(gid_t))
            .unwrap_or_else(|| get_group_by_name(g_str).map(|group| group.gid()))
    })
}

fn start_process(spec: &ModuleSpec) -> Result<Child, Error> {
    //tokio command returns a result of a ChildFuture
    let settings = spec.config().settings();
    // where should the files and paths be cooked?
    // see tokio::fs::canonicalize
    let mut cmd = Command::new(settings.exe());
    cmd.current_dir(settings.working_directory());
    if let Some(cmd_args) = settings.exe_args() {
        cmd.args(cmd_args);
    }
    if let Some(args) = settings.args() {
        cmd.args(args);
    }
    if let Some(user) = lookup_uid(settings.user()) {
        println!("Running as uid: {}", user);
        cmd.uid(user);
    }
    if let Some(group) = lookup_gid(settings.group()) {
        println!("Running as gid: {}", group);
        cmd.gid(group);
    }
    if let Some(envs) = spec.config().env() {
        for env in envs {
            cmd.env(env.key(), env.value());
        }
    }
    if let Some(stderr) = settings.stderr_log() {
        let file = File::create(stderr).map_err(|_| Error::from(ErrorKind::FileOpen))?;
        cmd.stderr(file);
    }
    if let Some(stdout) = settings.stdout_log() {
        let file = File::create(stdout).map_err(|_| Error::from(ErrorKind::FileOpen))?;
        cmd.stdout(file);
    }
    cmd.spawn().map_err(|_| Error::from(ErrorKind::ForkFailed))
}

fn kill_process(spec: &ModuleSpec) {
    if let Some(pid) = spec.status().and_then(|status| status.pid()) {
        let nix_pid = Pid::from_raw(*pid as i32);
        kill(nix_pid, Signal::SIGTERM).unwrap_or_else(|_e| {
            println!("kill failed");
        });
    }
}

fn db_update(db: Arc<DB>, module_name: String, spec: &ModuleSpec) -> Result<(), Error> {
    {
        let mut lock = db.lock().map_err(|_| Error::from(ErrorKind::DbLock))?;
        lock.insert(&module_name, spec)
            .map_err(|_| Error::from(ErrorKind::DbInsert))?;
    }
    db.flush().map_err(|_| Error::from(ErrorKind::DbFlush))
}

pub async fn control_loop(
    module_name: String,
    db: Arc<DB>,
    mut control_rx: mpsc::Receiver<ControlMessage>,
    mut response: mpsc::Sender<ControlResponse>,
) {
    let mut restart_count = 0;
    // loop until module is removed
    loop {
        // Get database entry for module.
        let spec: Result<ModuleSpec, _> = db
            .retrieve(&module_name)
            .map_err(|_err| Error::from(ErrorKind::DbRetrieve));
        if let Ok(spec) = spec {
            // fork, setup and exec process
            if None == spec.status().and_then(|status| status.pid()) {
                println!("Starting Module {}", module_name);
                // If child process doesn't exists, end it.
                //Hold off here, for now increment restart_count
                restart_count += 1;
                let result = start_process(&spec);
                match result {
                    Ok(child) => {
                        let spec = spec.with_status(Status::new(Some(child.id()), None));
                        // save Started status & pid in database.
                        db_update(db.clone(), module_name.to_string(), &spec)
                            .expect("Write to database failed");
                        let db = db.clone();
                        let module_name = module_name.to_string();
                        // This is a join handle here. How do I wait on the Child
                        tokio::spawn(async {
                            let status = child.await.expect("child process encountered an error");
                            let spec = spec.with_status(Status::new(None, status.code()));
                            db_update(db, module_name, &spec).expect("Write to database failed");
                            println!("child status was: {}", status);
                        });
                    }
                    Err(e) => {
                        println!("Process failed to start {}", e);
                        let db = db.clone();
                        let spec = spec.with_status(Status::new(None, None));
                        db_update(db, module_name.to_string(), &spec)
                            .expect("Write to database failed");
                    }
                }
            }
        } else {
            println!("Failed to retrieve module");
        }

        if restart_count > 10 {
            break;
        }

        // Wait on PID or channel update.
        match control_rx.try_recv() {
            Ok(_msg) => {
                println!("Module {} recoved a control message", module_name);
                let spec: Result<ModuleSpec, _> = db
                    .retrieve(&module_name)
                    .map_err(|_err| Error::from(ErrorKind::DbRetrieve));
                if let Ok(spec) = spec {
                    kill_process(&spec);
                }
                break;
            }
            Err(mpsc::error::TryRecvError::Closed) => {
                println!("Module {} has a closed control channel", module_name);
                break;
            }
            Err(mpsc::error::TryRecvError::Empty) => {}
        };

        time::delay_for(time::Duration::from_secs(2)).await;
    }
    response
        .send(ControlResponse::Stopped(module_name))
        .await
        .map_err(|_| Error::from(ErrorKind::InitializeTokio))
        .expect("send failed");
}
