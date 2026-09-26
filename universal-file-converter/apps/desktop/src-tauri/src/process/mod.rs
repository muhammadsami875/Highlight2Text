use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

pub struct ProcessRunner {
    pub executable: PathBuf,
    pub args: Vec<String>,
    pub working_dir: Option<PathBuf>,
    pub env: HashMap<String, String>,
    pub timeout: Duration,
}

pub struct ProcessOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl ProcessRunner {
    pub fn new(executable: PathBuf) -> Self {
        Self {
            executable,
            args: Vec::new(),
            working_dir: None,
            env: HashMap::new(),
            timeout: Duration::from_secs(300),
        }
    }

    pub fn args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    pub fn working_dir(mut self, dir: PathBuf) -> Self {
        self.working_dir = Some(dir);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn env(mut self, key: String, value: String) -> Self {
        self.env.insert(key, value);
        self
    }

    pub fn run(&self) -> Result<ProcessOutput, String> {
        let mut cmd = std::process::Command::new(&self.executable);
        cmd.args(&self.args);

        if let Some(ref dir) = self.working_dir {
            cmd.current_dir(dir);
        }

        for (k, v) in &self.env {
            cmd.env(k, v);
        }

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to execute {}: {}", self.executable.display(), e))?;

        Ok(ProcessOutput {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}
