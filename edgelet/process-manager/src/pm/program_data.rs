use serde_derive::{Deserialize, Serialize};
#[allow(unused_imports)]
use serde_json::Value;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct EnvVar {
    #[serde(rename = "key")]
    key: String,
    #[serde(rename = "value")]
    value: String,
}

impl EnvVar {
    pub fn new(key: String, value: String) -> Self {
        EnvVar { key, value }
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ProcessParameters {
    command: Vec<String>,
    args: Vec<String>,
    working_directory: String,
    user: Option<String>,
    group: Option<String>,
    stderr_log: Option<String>,
    stdout_log: Option<String>,
}

impl ProcessParameters {
    pub fn new(command: Vec<String>, args: Vec<String>, working_directory: String) -> Self {
        ProcessParameters {
            command,
            args,
            working_directory,
            ..ProcessParameters::default()
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Config {
    env: Option<Vec<EnvVar>>,
    settings: ProcessParameters,
}

impl Config {
    pub fn new(env: Option<Vec<EnvVar>>, settings: ProcessParameters) -> Self {
        Config { env, settings }
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Status {
    pid: Option<u32>,
    exit_status: Option<i32>,
}

impl Status {
    pub fn new(pid: Option<u32>, exit_status: Option<i32>) -> Self {
        Status { pid, exit_status }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ModuleSpec {
    name: String,
    type_: String,
    config: Config,
    status: Option<Status>,
}

impl ModuleSpec {
    pub fn new(name: String, config: Config) -> Self {
        ModuleSpec {
            name: name,
            type_: "native".to_string(),
            config: config,
            status: None,
        }
    }
    pub fn with_status(mut self, status: Status) -> Self {
        self.status = Some(status);
        self
    }
}
