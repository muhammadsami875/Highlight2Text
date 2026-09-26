use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

#[derive(Debug)]
pub struct ProcessOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

impl ProcessOutput {
    pub fn success(&self) -> bool {
        self.exit_code == 0 && !self.timed_out
    }
}

pub struct ProcessRunner {
    executable: PathBuf,
    args: Vec<String>,
    working_dir: Option<PathBuf>,
    env: HashMap<String, String>,
    timeout: Duration,
    stdin_data: Option<Vec<u8>>,
}

impl ProcessRunner {
    pub fn new(executable: PathBuf) -> Self {
        Self {
            executable,
            args: Vec::new(),
            working_dir: None,
            env: HashMap::new(),
            timeout: Duration::from_secs(300),
            stdin_data: None,
        }
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
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

    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    pub fn stdin_data(mut self, data: Vec<u8>) -> Self {
        self.stdin_data = Some(data);
        self
    }

    pub fn run(&self) -> Result<ProcessOutput, String> {
        let mut cmd = std::process::Command::new(&self.executable);
        cmd.args(&self.args);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        if self.stdin_data.is_some() {
            cmd.stdin(Stdio::piped());
        } else {
            cmd.stdin(Stdio::null());
        }

        if let Some(ref dir) = self.working_dir {
            cmd.current_dir(dir);
        }

        for (k, v) in &self.env {
            cmd.env(k, v);
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn {}: {}", self.executable.display(), e))?;

        if let Some(ref data) = self.stdin_data {
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                let _ = stdin.write_all(data);
            }
        }

        let start = std::time::Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    let stdout = child
                        .stdout
                        .take()
                        .map(|mut r| {
                            let mut s = String::new();
                            use std::io::Read;
                            let _ = r.read_to_string(&mut s);
                            s
                        })
                        .unwrap_or_default();

                    let stderr = child
                        .stderr
                        .take()
                        .map(|mut r| {
                            let mut s = String::new();
                            use std::io::Read;
                            let _ = r.read_to_string(&mut s);
                            s
                        })
                        .unwrap_or_default();

                    return Ok(ProcessOutput {
                        exit_code: status.code().unwrap_or(-1),
                        stdout,
                        stderr,
                        timed_out: false,
                    });
                }
                Ok(None) => {
                    if start.elapsed() > self.timeout {
                        let _ = child.kill();
                        let _ = child.wait();
                        return Ok(ProcessOutput {
                            exit_code: -1,
                            stdout: String::new(),
                            stderr: format!(
                                "Process timed out after {}s",
                                self.timeout.as_secs()
                            ),
                            timed_out: true,
                        });
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(e) => {
                    return Err(format!("Error waiting for process: {}", e));
                }
            }
        }
    }
}

pub fn detect_engine(name: &str, version_flag: &str) -> Option<String> {
    let output = ProcessRunner::new(PathBuf::from(name))
        .arg(version_flag)
        .timeout(Duration::from_secs(10))
        .run();

    match output {
        Ok(o) if o.success() => {
            let version = o.stdout.lines().next().unwrap_or("").trim().to_string();
            Some(version)
        }
        Ok(o) if !o.stdout.is_empty() => {
            Some(o.stdout.lines().next().unwrap_or("").trim().to_string())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_echo() {
        let output = ProcessRunner::new(PathBuf::from("echo"))
            .arg("hello")
            .arg("world")
            .run()
            .unwrap();

        assert!(output.success());
        assert_eq!(output.stdout.trim(), "hello world");
        assert!(!output.timed_out);
    }

    #[test]
    fn run_false_returns_nonzero() {
        let output = ProcessRunner::new(PathBuf::from("false"))
            .run()
            .unwrap();

        assert!(!output.success());
        assert_ne!(output.exit_code, 0);
    }

    #[test]
    fn run_captures_stderr() {
        let output = ProcessRunner::new(PathBuf::from("sh"))
            .arg("-c")
            .arg("echo error >&2")
            .run()
            .unwrap();

        assert!(output.stderr.contains("error"));
    }

    #[test]
    fn run_with_timeout() {
        let output = ProcessRunner::new(PathBuf::from("sleep"))
            .arg("10")
            .timeout(Duration::from_millis(200))
            .run()
            .unwrap();

        assert!(output.timed_out);
        assert!(!output.success());
    }

    #[test]
    fn run_nonexistent_binary() {
        let result = ProcessRunner::new(PathBuf::from("nonexistent_binary_xyz"))
            .run();

        assert!(result.is_err());
    }

    #[test]
    fn run_with_env() {
        let output = ProcessRunner::new(PathBuf::from("sh"))
            .arg("-c")
            .arg("echo $TEST_VAR")
            .env("TEST_VAR", "hello_from_env")
            .run()
            .unwrap();

        assert!(output.success());
        assert_eq!(output.stdout.trim(), "hello_from_env");
    }

    #[test]
    fn run_with_working_dir() {
        let output = ProcessRunner::new(PathBuf::from("pwd"))
            .working_dir(PathBuf::from("/tmp"))
            .run()
            .unwrap();

        assert!(output.success());
        assert!(output.stdout.trim().starts_with("/tmp"));
    }

    #[test]
    fn detect_engine_echo() {
        let version = detect_engine("echo", "test");
        assert!(version.is_some());
    }

    #[test]
    fn detect_engine_nonexistent() {
        let version = detect_engine("nonexistent_engine_xyz", "--version");
        assert!(version.is_none());
    }
}
