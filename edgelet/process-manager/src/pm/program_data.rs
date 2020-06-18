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
    pub fn with_user(mut self, user: Option<String>) -> Self {
        self.user = user;
        self
    }
    pub fn with_group(mut self, group: Option<String>) -> Self {
        self.group = group;
        self
    }
    pub fn with_logs(mut self, stderr_log: Option<String>, stdout_log: Option<String>) -> Self {
        self.stderr_log = stderr_log;
        self.stdout_log = stdout_log;
        self
    }

    pub fn exe(&self) -> &String {
        &self.command[0]
    }
    pub fn exe_args(&self) -> Option<&[String]> {
        if self.command.len() > 1 {
            Some(&self.command[1..])
        } else {
            None
        }
    }
    pub fn args(&self) -> Option<&[String]> {
        if self.args.len() > 0 {
            Some(&self.args[..])
        } else {
            None
        }
    }
    pub fn working_directory(&self) -> &str {
        self.working_directory.as_ref()
    }
    pub fn user(&self) -> Option<&str> {
        self.user.as_ref().map(String::as_str)
    }
    pub fn group(&self) -> Option<&str> {
        self.group.as_ref().map(String::as_str)
    }
    pub fn stderr_log(&self) -> Option<&str> {
        self.stderr_log.as_ref().map(String::as_str)
    }
    pub fn stdout_log(&self) -> Option<&str> {
        self.stdout_log.as_ref().map(String::as_str)
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

    pub fn env(&self) -> Option<&Vec<EnvVar>> {
        self.env.as_ref()
    }
    pub fn settings(&self) -> &ProcessParameters {
        &self.settings
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

    pub fn pid(&self) -> Option<&u32> {
        self.pid.as_ref()
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

    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn config(&self) -> &Config {
        &self.config
    }
    pub fn status(&self) -> Option<&Status> {
        self.status.as_ref()
    }
}
