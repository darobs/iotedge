// Copyright (c) Microsoft. All rights reserved.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use clap::crate_name;
use crossbeam::crossbeam_channel::{bounded, SendError, Sender};

use futures::future::{self, FutureResult};
use rustbreak::Database;

use crate::error::{Error, ErrorKind};
use crate::Command;

mod program_data;
mod runner;

#[derive(Default)]
pub struct PM {
    workingdir: String,
}

pub enum ControlMessage {
    Stop,
    Exit,
}

pub enum ControlResponse {
    Stopped,
}

//type ModuleSave = BTreeMap<String, program_data::ModuleSpec>;
type DB = Database<String>;

impl PM {
    pub fn new(workingdir: String) -> Self {
        PM { workingdir }
    }

    fn load_db(&self, workingdir: &str) -> Result<DB, Error> {
        let mut db_path: PathBuf = PathBuf::new();
        db_path.push(workingdir);
        db_path.push("processes.yaml");

        println!("Openinf db from {:?}", db_path);
        // Create a database in the workdir path
        DB::open(db_path).map_err(|err| Error::from(ErrorKind::DbOpen))
    }
}

impl Command for PM {
    type Future = FutureResult<(), Error>;

    #[allow(clippy::print_literal)]
    fn execute(self) -> Self::Future {
        println!("{} running in dir: {}", crate_name!(), self.workingdir,);

        future::done(self.load_db(&self.workingdir).and_then(|db| {
            let m1_name = "module1".to_string();
            let p1 = program_data::ProcessParameters::new(
                vec!["/bin/ls".to_string()],
                vec!["-l".to_string()],
                "/tmp".to_string(),
            );
            let e1 = program_data::EnvVar::new("A".to_string(), "B".to_string());
            let c1 = program_data::Config::new(Some(vec![e1]), p1);
            let m1 = program_data::ModuleSpec::new(m1_name.clone(), c1);
            db.insert(&m1_name, m1)
                .map_err(|err| Error::from(ErrorKind::DbInsert))?;

            let m2_name = "module2".to_string();
            let p2 = program_data::ProcessParameters::new(
                vec!["/bin/bash".to_string()],
                vec![
                    "-c".to_string(),
                    "while true; do date; sleep 10; done".to_string(),
                ],
                "/tmp".to_string(),
            );
            let c2 = program_data::Config::new(None, p2);
            let m2 = program_data::ModuleSpec::new(m2_name.clone(), c2);
            db.insert(&m2_name, m2)
                .map_err(|err| Error::from(ErrorKind::DbInsert))?;

            db.flush().map_err(|err| Error::from(ErrorKind::DbFlush))?;
            let module_spec: program_data::ModuleSpec = db
                .retrieve("module1")
                .map_err(|err| Error::from(ErrorKind::DbRetrieve))?;
            let module_spec: program_data::ModuleSpec = db
                .retrieve("module2")
                .map_err(|err| Error::from(ErrorKind::DbRetrieve))?;

            let (control_send, control_rx) = bounded(2);
            let (response_send, response_rx) = bounded(1);
            let db = Arc::new(db);
            thread::spawn(move || {
                runner::control_loop(m2_name, db.clone(), control_rx, response_send)
            });
            loop {
                match response_rx.recv() {
                    Ok(ControlResponse::Stopped) => {
                        break;
                    }
                    _ => panic!("Error receiving a ResultMsg."),
                }
            }
            Ok(())
        }))
        // Set up 1 or two long running processes.
        // Launch these tasks.
        //
    }
}
