#[macro_export]
macro_rules! git {
    ($($arg:expr),*) => { std::process::Command::new("git")$(.arg($arg))* };
}

mod shell;
mod types;

pub use {
    shell::{CommandExt, OutputExt, commit_file},
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
pub fn setup(tmp_dir: &'static str) -> (Test, PathBuf) {
    let mut t = Test::new(tmp_dir);

    // The place where we initialize the git history. Fill it out with events.
    let d_base = t.as_path().join("base");
    let d_base_git = d_base.join(".git");
    // The place where we'll make into a bare repo with the history from `base`.
    let d_repo = t.as_path().join("repo");

    fs::create_dir_all(&d_base).unwrap();
    fs::create_dir_all(&d_repo).unwrap();

    env::set_current_dir(&d_base).unwrap();
    git!("init").run();
    git!("branch", "-m", "main").run();
    commit_file(&mut t, "README.md");
    commit_file(&mut t, "file-1.txt");
    let c1 = git!("rev-parse", "HEAD").get_stdout();
    commit_file(&mut t, "file-2.txt");
    let c2 = git!("rev-parse", "HEAD").get_stdout();
    commit_file(&mut t, "file-3.txt");
    let c3 = git!("rev-parse", "HEAD").get_stdout();
    commit_file(&mut t, "last.txt");

    git!("checkout", "-b", "B1").run();
    git!("reset", "--hard", c1).run();

    git!("checkout", "-b", "B2").run();
    git!("reset", "--hard", c2).run();

    git!("checkout", "-b", "B3").run();
    git!("reset", "--hard", c3).run();

    git!("checkout", "main").run();

    fs::rename(&d_base_git, &d_repo).unwrap();
    git!("-C", &d_base_git, "config", "--bool", "core.bare", "true");
    fs::remove_dir_all(d_base).unwrap(); // Intentionally drop `d_base`
    env::set_current_dir(&d_repo).unwrap();

    git!("worktree", "add", "B1").run();
    git!("worktree", "add", "B2").run();
    git!("worktree", "add", "D3").run();
    git!("-C", "D3", "checkout", "B3").run();
    git!("branch", "-D", "D3").get();

    (t, d_repo)
}
