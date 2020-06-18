// Copyright (c) Microsoft. All rights reserved.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use clap::crate_name;
use tokio::sync;
use tokio::time;

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
    Stopped(String),
}

//type ModuleSave = BTreeMap<String, program_data::ModuleSpec>;
type DB = Database<String>;

impl PM {
    pub fn new(workingdir: String) -> Self {
        PM { workingdir }
    }

    fn load_db(&self) -> Result<DB, Error> {
        let mut db_path: PathBuf = PathBuf::new();
        db_path.push(&self.workingdir);
        db_path.push("processes.yaml");

        println!("Openinf db from {:?}", db_path);
        // Create a database in the workdir path
        DB::open(db_path).map_err(|err| Error::from(ErrorKind::DbOpen))
    }

    fn build_database(&self, db: &DB) -> Result<Vec<String>, Error> {
        let m1_name = "module1".to_string();
        let p1 = program_data::ProcessParameters::new(
            vec!["/bin/ls".to_string()],
            vec!["-l".to_string()],
            "/tmp".to_string(),
        )
        .with_logs(
            Some("/tmp/m1.err".to_string()),
            Some("/tmp/m1.out".to_string()),
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
        Ok(vec![m1_name, m2_name])
    }
}

#[async_trait]
impl Command for PM {
    #[allow(clippy::print_literal)]
    async fn execute(self) -> Result<(), Error> {
        println!("{} running in dir: {}", crate_name!(), self.workingdir,);

        let db = self.load_db()?;
        let modules = self.build_database(&db)?;
        let db = Arc::new(db);

        let (response_send, mut response_rx) = sync::mpsc::channel(1);
        let mut send_channels = Vec::new();
        for module in modules {
            let module_spec: program_data::ModuleSpec = db
                .retrieve(&module)
                .map_err(|err| Error::from(ErrorKind::DbRetrieve))?;
            let (mut control_send, mut control_rx) = sync::mpsc::channel(2);
            send_channels.push(control_send);
            tokio::spawn(runner::control_loop(
                module.to_string(),
                db.clone(),
                control_rx,
                response_send.clone(),
            ));
        }

        let mut modules_running = send_channels.len();

        loop {
            match response_rx.recv().await {
                Some(ControlResponse::Stopped(m)) => {
                    println!("Module {} Stopped", m);
                    modules_running = modules_running - 1;
                    if modules_running <= 0 {
                        break;
                    }
                }
                _ => panic!("Error receiving a ResultMsg."),
            }
            if modules_running == 1 {
                println!("Only one module left, waiting 1 sec and killing the last");
                time::delay_for(time::Duration::from_secs(1)).await;
                for ch in send_channels.iter_mut() {
                    ch.send(ControlMessage::Stop).await.unwrap_or_else(|e| {
                        println!("Attempt to stop chanel: {}", e);
                    });
                }
            }
        }
        Ok(())
    }
}
