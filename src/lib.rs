#[macro_export]
macro_rules! git {
    ($($arg:expr),*) => { std::process::Command::new("git")$(.arg($arg))* };
}

mod shell;
mod types;

pub use {
    shell::{CommandExt, OutputExt, cd, commit_file},
    types::Test,
};

use std::env;
use std::{fs, path::PathBuf};

/// Set up the test directory to a git repo of the state below, and returns the
/// path to the `repo/` dir here:
/// └── repo
///    ├── .git (bare repo)
///    ├── B1
///    │  ├── one
///    │  └── README.md
///    ├── B2
///    │  ├── one
///    │  ├── two
///    │  └── README.md
///    └── D3
///       ├── one
///       ├── two
///       ├── three
///       └── README.md
pub fn setup() -> (Test, PathBuf) {
    let mut t = Test::new("gco-test");

    // The place where we initialize the git history. Fill it out with events.
    let d_base = t.as_path().join("base");
    // The place where we'll make into a bare repo with the history from `base`.
    let d_repo = t.as_path().join("repo");

    fs::create_dir_all(&d_base).unwrap();
    fs::create_dir_all(&d_repo).unwrap();

    env::set_current_dir(&d_base).unwrap();
    git!("init", "--initial-branch=main").snw();
    git!("config", "--local", "user.email", "git@git.git").snw();
    git!("config", "--local", "user.name", "git").snw();

    eprintln!("=== Initialized a git repo ===");

    commit_file(&mut t, "README.md");
    commit_file(&mut t, "file-1.txt");

    git!("rev-parse", "--verify", "HEAD").snw();
    let c1 = git!("rev-parse", "--verify", "HEAD").get_stdout();
    commit_file(&mut t, "file-2.txt");

    git!("rev-parse", "--verify", "HEAD").snw();
    let c2 = git!("rev-parse", "--verify", "HEAD").get_stdout();
    commit_file(&mut t, "file-3.txt");

    git!("rev-parse", "--verify", "HEAD").snw();
    let c3 = git!("rev-parse", "--verify", "HEAD").get_stdout();
    commit_file(&mut t, "last.txt");

    {
        fn ok(sha: &str) -> bool {
            sha.is_ascii() && sha.len() == 40
        }
        assert!(ok(&c1), "Commit #1 is a strange one: {c1}");
        assert!(ok(&c2), "Commit #2 is a strange one: {c2}");
        assert!(ok(&c3), "Commit #3 is a strange one: {c3}");
    }

    git!("checkout", "-b", "B1").snw();
    git!("reset", "--hard", c1).snw();
    assert_eq!(git!("branch", "--show-current").get_stdout(), "B1");

    git!("checkout", "-b", "B2").snw();
    git!("reset", "--hard", c2).snw();
    assert_eq!(git!("branch", "--show-current").get_stdout(), "B2");

    git!("checkout", "-b", "B3").snw();
    git!("reset", "--hard", c3).snw();
    assert_eq!(git!("branch", "--show-current").get_stdout(), "B3");

    git!("checkout", "main").snw();
    assert_eq!(git!("branch", "--show-current").get_stdout(), "main");

    git!("-C", d_base.join(".git"), "config", "--bool", "core.bare", "true");
    fs::rename(d_base.join(".git"), &d_repo).unwrap();
    fs::remove_dir_all(d_base).unwrap(); // Intentionally drop `d_base`
    env::set_current_dir(&d_repo).unwrap();

    git!("worktree", "add", "B1").snw();
    git!("worktree", "add", "B2").snw();
    git!("worktree", "add", "D3").snw();
    git!("-C", d_repo.join("D3"), "checkout", "B3").snw();
    git!("branch", "-D", "D3").snw();

    (t, d_repo)
}
