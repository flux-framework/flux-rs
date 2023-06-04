use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::{env, str};

use lazy_static::lazy_static;

pub struct SideFlux {
    pub uri: String,
    pub child: Child,
}

lazy_static! {
    static ref ENV_SET: Mutex<bool> = Mutex::new(false);
}

fn set_flux_env() {
    // This function is required to ensure that things like FLUX_CONNECTOR_PATH
    // are set before attempting to connect into a Flux instance with the local
    // or ssh connector.

    // Since the output of `flux env` changes with the state of the current
    // environment, we want to ensure that we only modify the env once for
    // the process.

    let mut env_set = ENV_SET.lock().unwrap();
    if *env_set {
        return;
    }

    let output = Command::new("flux")
        .args(&["env"])
        .output()
        .expect("Flux failed to start");
    let stdout = str::from_utf8(&output.stdout).unwrap();

    for line in stdout.lines() {
        // TODO: replace with strip_prefix and split_once once they are stable
        assert!(line.starts_with("export "));
        let key_value = &line[7..line.len()];
        let delim_idx = key_value.find('=').expect("Missing '=' in flux env output");
        let (key, mut value) = key_value.split_at(delim_idx);
        value = &value[2..value.len() - 1]; // remove '=' and '"'s
        env::set_var(key, value);
    }

    *env_set = true;
}

impl SideFlux {
    pub fn start(size: u32) -> SideFlux {
        let mut child = Command::new("flux")
            .args(&[
                "start",
                format!("--test-size={}", size).as_str(),
                "--",
                "bash",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Flux failed to start");

        {
            let stdin = child
                .stdin
                .as_mut()
                .expect("Failed to take stdin from sideflux");
            stdin
                .write_all("echo READY\n".as_bytes())
                .expect("Failed to write to stdin");
            stdin
                .write_all("printenv FLUX_URI\n".as_bytes())
                .expect("Failed to write to stdin");
        }

        let stdout_handle = child
            .stdout
            .take()
            .expect("Failed to take stdout from sideflux");
        let mut stdout = BufReader::new(stdout_handle);
        let mut line = String::new();
        stdout
            .read_line(&mut line)
            .expect("Failed to read line from stdout");
        let mut trimmed_line = line.trim_end().to_string();
        assert_eq!(trimmed_line, "READY");

        line.clear();
        stdout
            .read_line(&mut line)
            .expect("Failed to read line from stdout");
        trimmed_line = line.trim_end().to_string();
        println!("Using sideflux with URI: {}", trimmed_line);

        set_flux_env();

        SideFlux {
            uri: trimmed_line,
            child,
        }
    }
}

impl Drop for SideFlux {
    fn drop(&mut self) {
        {
            let stdin = self
                .child
                .stdin
                .as_mut()
                .expect("Failed to open stdin to sideflux");
            stdin
                .write_all("exit".as_bytes())
                .expect("Failed to write to stdin");
        }
        // TODO: should first try SIGTERM before SIGKILL
        self.child.kill().expect("Could not kill Flux instance");
    }
}
