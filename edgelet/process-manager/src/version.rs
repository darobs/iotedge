// Copyright (c) Microsoft. All rights reserved.

use async_trait::async_trait;
use clap::crate_name;

use crate::error::Error;
use crate::Command;

#[derive(Default)]
pub struct Version;

impl Version {
    pub fn new() -> Self {
        Version
    }
}

#[async_trait]
impl Command for Version {
    #[allow(clippy::print_literal)]
    async fn execute(self) -> Result<(), Error> {
        println!(
            "{} {}",
            crate_name!(),
            edgelet_core::version_with_source_version(),
        );
        Ok(())
    }
}
