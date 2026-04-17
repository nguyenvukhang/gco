use git_checkout2::*;

use std::env;

#[test]
fn setup_test_branch_1() {
    let (_t, root) = setup();
    assert_eq!(
        git!("-C", root.join("B1"), "branch", "--show-current").get_stdout(),
        "B1"
    )
}

#[test]
fn setup_test_branch_2() {
    let (_t, root) = setup();
    assert_eq!(
        git!("-C", root.join("B2"), "branch", "--show-current").get_stdout(),
        "B2"
    )
}

#[test]
fn setup_test_branch_3() {
    let (_t, root) = setup();
    assert_eq!(
        git!("-C", root.join("D3"), "branch", "--show-current").get_stdout(),
        "B3"
    )
}

/// Jump from the lift-lobby (git workspace area, but not in any git workspace)
#[test]
fn t1() {
    let (_t, root) = setup();
    env::set_current_dir(&root).unwrap();
    let output = git!("checkout2", "B1").get();
    assert_eq!(cd(output.stdout), cd(root.join("B1")));
    assert_eq!(output.status.code(), Some(64));
}

/// Jump using ref, from B1 -> B2. Expected to parse:
/// fatal: 'B2' is already used by worktree at '/tmp/gco/repo/B2'
#[test]
fn t2() {
    let (_t, root) = setup();
    env::set_current_dir(root.join("B1")).unwrap();
    let output = git!("checkout2", "B2").get();
    assert_eq!(cd(output.stdout), cd(root.join("B2")));
    assert_eq!(output.status.code(), Some(64));
}

/// Jump using ref, from B1 -> B3, but where the directory doesn't match the
/// branch name:
/// fatal: 'B3' is already used by worktree at '/tmp/gco/repo/D3'
#[test]
fn t3() {
    let (_t, root) = setup();
    env::set_current_dir(root.join("B1")).unwrap();
    let output = git!("checkout2", "B3").get();
    assert_eq!(cd(output.stdout), cd(root.join("D3")));
    assert_eq!(output.status.code(), Some(64));
}

/// Jump using directory, from B1 -> B3, but we use D3 as the target instead
/// of B3.
#[test]
fn t4() {
    let (_t, root) = setup();
    env::set_current_dir(root.join("B1")).unwrap();
    let output = git!("checkout2", "D3").get();
    assert_eq!(cd(output.stdout), cd(root.join("D3")));
    assert_eq!(output.status.code(), Some(64));
}

/// `git-checkout2` should return the same exit code as `git checkout` in an
/// empty repository.
#[test]
fn empty_directory() {
    let t = Test::new("gco-test");
    let _ = std::fs::remove_dir_all(t.as_path()).unwrap();
    std::fs::create_dir(t.as_path()).unwrap();
    env::set_current_dir(t.as_path()).unwrap();
    let lhs = git!("checkout2", "zeno").get();
    let rhs = git!("checkout", "zeno").get();
    assert_eq!(lhs.status, rhs.status);
}

/// `git-checkout2` should return the same exit code as `git checkout` when a
/// branch doesn't exist.
#[test]
fn branch_not_exists() {
    let (_t, root) = setup();
    env::set_current_dir(root.join("B1")).unwrap();
    let lhs = git!("checkout2", "zeno").get();
    let rhs = git!("checkout", "zeno").get();
    assert_eq!(lhs.status, rhs.status);
}
