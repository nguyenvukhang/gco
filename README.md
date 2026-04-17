# git checkout2

An opinionated wrapper around `git checkout` that helps you jump where you need
to go.

## The Pitch

Consider this: you're working with multiple
[worktrees](https://git-scm.com/docs/git-worktree), and you type `git checkout
dev`. The expected next state is that your current directory is populated with
files at `dev`, and your probably want to get straight to making edits on `dev`.
However, if `dev` is already checked out in another worktree, then `git
checkout` will fail. Wouldn't it be nice if we could just parse that message on
stderr and jump straight to that worktree?

## How to use `git-checkout2`

1. Install `git-checkout2` with

   ```
   make configure
   make install
   ```

   (you can change the installation directory in `CMakeLists.txt`)

2. In your `.bashrc` or `.zshrc`, add simple function to wrap `git-checkout2`.
   This is because running any binary directly will never be able to change your
   current directory in the interactive shell. Here's a suggested
   (POSIX-compliant) function:
   ```sh
   gco() {
     TARGET=$(git checkout2 $1)
     EXIT_CODE=$?
     if [ $EXIT_CODE -eq 64 ]; then
       cd $TARGET
     fi
     unset TARGET
     return $EXIT_CODE
   }
   ```

## The Promise

`git-checkout2` does two very simple things. For this discussion, assume that we
setup `gco` like the above, and we run the shell command `gco GOAL`.

1. If the `GOAL` branch is already used in another worktree, then we'll be
   transported there with the `cd` command.

2. If the first condition fails to find matches, and if somehow there is a
   worktree whose directory name is `GOAL`, then we'll be transported there
   instead.
   - This is useful when you have more branches than worktrees, and the
     worktrees are no longer pointing to their original respective branches.

3. It does all this with zero heap allocation.

## The Boundaries

In the spirit of simplicity (this project is already an over-engineered shell
script), `git-checkout2` will only accept one shell argument. That is, if more
than 1 argument (or no arguments) is supplied to `git-checkout2`, then it will
simply forward all those arguments to `git checkout`. This eases the load of
argument parsing.

## Testing

Tests are written in Rust, since I do feel like that's the best language for
ensuring correctness. These can be found in `tests/`.

## Nerding out

This was originally nothing more than a long-winded zsh function living in my
`.zshrc`. But at some point I wanted to change some behavior, and I found it
very hard to read zsh code I've written a long time ago. So I rewrote it in
Rust. But Rust's standard library API for `Command` and piping forced me to
heap-allocate, which I just didn't want. And so since this project is small
enough, I wrote it all in C and here we are. I'm pretty happy about it, despite
its simplicity. Thanks for reading, and happy hacking!
