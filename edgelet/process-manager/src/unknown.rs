// Copyright (c) Microsoft. All rights reserved.

use async_trait::async_trait;

use crate::error::{Error, ErrorKind};
use crate::Command;

pub struct Unknown {
    command: String,
}

impl Unknown {
    pub fn new(command: String) -> Self {
        Unknown { command }
    }
}

#[async_trait]
impl Command for Unknown {
    async fn execute(self) -> Result<(), Error> {
        eprintln!("unknown command: {}", self.command);
        Err(Error::from(ErrorKind::UnknownCommand))
    }
}
