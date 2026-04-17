# git checkout2

An opinionated wrapper around `git checkout`. If the checkout command worked
then nothing is done. Otherwise, this will try to locate the branch among all
the existing worktrees, and then print that directory to stdout so that the
shell can `cd` to it.

## More on `git-checkout2`

Consider this: you're working with multiple [git
worktrees](https://git-scm.com/docs/git-worktree), and you run a command like
`git checkout dev`. The expected next state is that your current directory is
populated with files from the commit which `dev` points to, and your expected
next command is probably to make edits to files in that state. However, if `dev`
is already checked out in another git worktree, then `git` will stop you and
tell you that. Won't it be nice if we could just parse that error message and
`cd` straight to that worktree? That's what this project is for.

It does two very simple things.

1. If the branch you're checking out is already used in another worktree, then
   `git-checkout2` will print the directory to that worktree to `stdout`, and
   then exit with an exit code of 64.
2. If somehow there is a git worktree whose directory name is the same as the
   name of the branch you're trying to check out, `git-checkout2` will again
   print that directory to stdout and exit with 64.

The goal of returning an exit code of 64 is to tell the shell when to execute
`cd`. Otherwise, `git-checkout2` aims to replicate the same exit code as running
`git-checkout` would.

Moreover, in the spirit of simplicity, `git-checkout2` will only accept one
shell argument. We request that other cases are handled by the user's shell
config. So then, here's the suggested zsh shell function to wrap `git-checkout2`:

```zsh
gco() {
  if [ $# -gt 1 ] || [ $# -eq 0 ]; then
    git checkout $@
    return
  fi
  TARGET=$(git checkout2 $1)
  local EC=$?
  if [ $EC -eq 64 ]; then
    cd $TARGET
  fi
  unset TARGET
  return $EC
}
```

## Nerding out

I originally had this bashed out in pure zsh, but I wanted to add tests and
change some behaviours (to become what it is today). So I rewrote it in Rust,
and since I've spent that effort I wanted to see if I could have full control
over things like memory allocation. And so I ended up rewriting it, one last
time, in C, while keeping the tests in Rust. As it stands right now, there is
zero heap memory allocated in `git-checkout2` 😎. Have a nice day and happy
hacking!
