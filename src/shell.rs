use crate::Test;
use std::{
    fs::File,
    io::Write,
    process::{Command, ExitStatus, Output, Stdio},
};

#[derive(Debug)]
pub struct Output2 {
    pub stdout: String,
    pub stderr: String,
    pub status: ExitStatus,
}

pub fn commit_file(t: &mut Test, pathspec: &'static str) {
    let mut f =
        File::options().create(true).append(true).open(pathspec).unwrap();
    writeln!(f, "data({})", t.id()).unwrap();
    git!("add", pathspec).snw();
    git!("commit", "-m", format!("Updated \"{pathspec}\"")).snw();
}

pub trait CommandExt {
    fn run(&mut self) -> Output;
    fn get_stdout(&mut self) -> String;
    fn get(&mut self) -> Output2;
    /// Spawn and wait.
    fn snw(&mut self);
}

impl CommandExt for Command {
    fn run(&mut self) -> Output {
        self.output().unwrap()
    }

    fn snw(&mut self) {
        let output = self.get();
        println!("{}", output.stderr);
        println!("{}", output.stdout);
    }

    fn get_stdout(&mut self) -> String {
        let output = self.stdout(Stdio::piped()).output().unwrap();
        let stdout = core::str::from_utf8(&output.stdout).unwrap();
        stdout.trim().to_string()
    }

    fn get(&mut self) -> Output2 {
        let output = self
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .unwrap();
        let stdout = core::str::from_utf8(&output.stdout).unwrap();
        let stderr = core::str::from_utf8(&output.stderr).unwrap();
        Output2 {
            stdout: stdout.trim().to_string(),
            stderr: stderr.trim().to_string(),
            status: output.status,
        }
    }
}

pub trait OutputExt {
    fn stdout(&self) -> &str;
    fn stderr(&self) -> &str;
}

impl OutputExt for Output {
    fn stdout(&self) -> &str {
        core::str::from_utf8(&self.stdout).unwrap()
    }
    fn stderr(&self) -> &str {
        core::str::from_utf8(&self.stderr).unwrap()
    }
}
