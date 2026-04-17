use git_checkout2::*;

use std::env;
use std::path::{Path, PathBuf};

fn cd<P: AsRef<Path>>(dir: P) -> PathBuf {
    let cwd = env::current_dir().unwrap();
    env::set_current_dir(dir).unwrap();
    let result = env::current_dir().unwrap();
    env::set_current_dir(cwd).unwrap();
    result
}

#[test]
fn setup_test_branch_1() {
    let (_t, d_repo) = setup("x");
    let output =
        git!("-C", d_repo.join("B1"), "branch", "--show-current").get();
    assert_eq!(output.stdout.trim(), "B1")
}

#[test]
fn setup_test_branch_2() {
    let (_t, d_repo) = setup("x");
    let output =
        git!("-C", d_repo.join("B2"), "branch", "--show-current").get();
    assert_eq!(output.stdout.trim(), "B2")
}

#[test]
fn setup_test_branch_3() {
    let (_t, d_repo) = setup("x");
    let output =
        git!("-C", d_repo.join("D3"), "branch", "--show-current").get();
    assert_eq!(output.stdout.trim(), "B3")
}

/// Jump from the lift-lobby (git workspace area, but not in any git workspace)
#[test]
fn t1() {
    let (_t, d_repo) = setup("t1");
    env::set_current_dir(&d_repo).unwrap();
    let output = git!("checkout2", "B1").get();
    eprintln!("[git-checkout2 stdout]:\n{}", output.stdout);
    eprintln!("[git-checkout2 stderr]:\n{}", output.stderr);

    let lhs = cd(output.stdout.trim());
    let rhs = cd(d_repo.join("B1"));
    assert_eq!(lhs, rhs);
    assert_eq!(output.status.code(), Some(64));
}

/// Jump using ref, from B1 -> B2. Expected to parse:
/// fatal: 'B2' is already used by worktree at '/tmp/gco/repo/B2'
#[test]
fn t2() {
    let (_t, d_repo) = setup("t2");
    env::set_current_dir(d_repo.join("B1")).unwrap();
    let output = git!("checkout2", "B2").get();
    eprintln!("[git-checkout2 stdout]:\n{}", output.stdout);
    eprintln!("[git-checkout2 stderr]:\n{}", output.stderr);

    let lhs = cd(output.stdout.trim());
    let rhs = cd(d_repo.join("B2"));
    assert_eq!(lhs, rhs);
    assert_eq!(output.status.code(), Some(64));
}

/// Jump using ref, from B1 -> B3, but where the directory doesn't match the
/// branch name:
/// fatal: 'B3' is already used by worktree at '/tmp/gco/repo/D3'
#[test]
fn t3() {
    let (_t, d_repo) = setup("t3");
    env::set_current_dir(d_repo.join("B1")).unwrap();
    let output = git!("checkout2", "B3").get();
    eprintln!("[git-checkout2 stdout]:\n{}", output.stdout);
    eprintln!("[git-checkout2 stderr]:\n{}", output.stderr);

    let lhs = cd(output.stdout.trim());
    let rhs = cd(d_repo.join("D3"));
    assert_eq!(lhs, rhs);
    assert_eq!(output.status.code(), Some(64));
}

/// Jump using directory, from B1 -> B3, but we use D3 as the target instead
/// of B3.
#[test]
fn t4() {
    let (_t, d_repo) = setup("t4");
    env::set_current_dir(d_repo.join("B1")).unwrap();
    let output = git!("checkout2", "D3").get();
    eprintln!("[git-checkout2 stdout]:\n{}", output.stdout);
    eprintln!("[git-checkout2 stderr]:\n{}", output.stderr);

    let lhs = cd(output.stdout.trim());
    let rhs = cd(d_repo.join("D3"));
    assert_eq!(lhs, rhs);
    assert_eq!(output.status.code(), Some(64));
}
