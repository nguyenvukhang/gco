// STDOUT is reserved for printing the target directory.

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

#define GIT_CHECKOUT_BUF_SZ 512
#define GIT_WORKTREE_BUF_SZ 2048

// #define DEBUG
#ifdef DEBUG
#define debug_printf(format, ...)                                              \
  fprintf(stderr, "[\x1b[32mINFO\x1b[m] " format "\n", __VA_ARGS__)
#else
#define debug_printf(...)
#endif

#define MAX(A, B) (A < B ? B : A)
#define MIN(A, B) (A < B ? A : B)

#define ERR(msg) write(STDERR_FILENO, msg "\n", sizeof(msg) + 1)

static char *GIT;

// Set the `GIT` variable.
void setup_git_binary() {
  GIT = getenv("GIT");
  if (GIT == NULL) {
    GIT = "git";
  }
}

// Returns 64 if this function's stdout output is meant to be taken as the
// target directory for the `cd` command.
int main(int argc, const char *argv[]) {
  debug_printf("\x1b[33mDEBUG MODE\x1b[m", 0);

  // Make sure that we only have one argument. Where got time to handle
  // comprehensive argument parsing.
  if (argc != 2) {
    ERR("git-checkout2 expects exactly 1 argument.");
    ERR("In exchange, it will do its best to locate this target for you.");
    return 1;
  }

#define GOAL argv[1]
  setup_git_binary();
  debug_printf("GIT = %s", GIT);

  int fd_checkout[2];
  if (pipe(fd_checkout) == -1) {
    ERR("pipe failed. This is necessary to read `git checkout` output.");
    return 1;
  }

  pid_t pid_checkout = fork();

  if (pid_checkout == -1) {
    ERR("fork failed. This is necessary to run `git checkout`.");
    return 1;
  } else if (pid_checkout == 0) {
    /* Child process: `git checkout` */
    dup2(fd_checkout[1], STDERR_FILENO); // Pipe stderr to the write end.
    close(fd_checkout[0]);
    close(fd_checkout[1]);
    execlp(GIT, GIT, "checkout", GOAL, NULL);
  }
  close(fd_checkout[1]);

  /* Parent process. */

  int fd_worktree[2];
  if (pipe(fd_worktree) == -1) {
    ERR("pipe failed. This is necessary to read `git worktree` output.");
    return 1;
  }

  pid_t pid_worktree = fork();
  if (pid_worktree == -1) {
    ERR("fork failed. This is necessary to run `git worktree`.");
    return 1;
  } else if (pid_worktree == 0) {
    /* Child process: `git worktree` */
    dup2(fd_worktree[1], STDOUT_FILENO); // Pipe stdout this time.
    close(fd_worktree[0]);
    close(fd_worktree[1]);
    execlp(GIT, GIT, "worktree", "list", "--porcelain", NULL);
  }
  close(fd_worktree[1]);

  char z[GIT_CHECKOUT_BUF_SZ], *c_left, *c_right, *c_line;
  int exit_code;

  waitpid(pid_checkout, &exit_code, 0);
  exit_code = exit_code == 0 ? 0 : 1;

  // By construction, zlen < GIT_CHECKOUT_BUF_SZ;
  const int zlen = read(fd_checkout[0], z, GIT_CHECKOUT_BUF_SZ - 1);
  debug_printf("Bytes read from `git checkout`: %d", zlen);
  z[zlen] = '\0';
  if (zlen == GIT_CHECKOUT_BUF_SZ) {
    debug_printf("Warning: possibly missing data from `git checkout` due to\n"
                 "insufficient buffer size.%s",
                 "");
  }

  debug_printf("GIT CHECKOUT OUTPUT:\n%s", z);

  // If all goes well, just reflect `git checkout's` stderr output back to the
  // terminal's stderr.
  if (exit_code == 0) {
    write(STDERR_FILENO, z, zlen);
    debug_printf("Everything went well, nothing to do anymore.", 0);
    return exit_code;
  }

  // If it turns out we're not even in a git repository, then exit early.
  if (strncmp(z, "fatal: not a git repository", 27) == 0) {
    write(STDERR_FILENO, z, zlen);
    debug_printf("Not in a git repo", 0);
    return exit_code;
  }

  // `git checkout` sometimes would override local changes, in which case the
  // error message is:
  // ```
  // error: Your local changes to the following files would be overwritten by
  // checkout:
  //   ...
  // Please commit your changes or stash them before you switch branches.
  // Aborting
  // ```
  if (strncmp(z, "error: Your local changes t", 27) == 0) {
    write(STDERR_FILENO, z, zlen);
    debug_printf("Your local changes (exit code: %d)", exit_code);
    return exit_code;
  }

  // If we see that this branch is already used at another worktree, we print
  // the directory to that worktree to STDOUT so the parent shell can go there.
  if (strncmp(z, "fatal:", 6) == 0) {
    c_left = strstr(z, "is already used by worktree at");
    if (c_left != NULL) {
      if ((c_left = strchr(c_left + 30, '\'')) == NULL) {
        ERR("Parsing error for \"is already used by worktree at...\"");
      }
      if ((c_right = strchr(++c_left, '\'')) == NULL) {
        ERR("Parsing error for \"is already used by worktree at...\"");
      }
      // OUTPUT ==========
      write(STDOUT_FILENO, c_left, c_right - c_left);
      return 64;
    }
  }

  // Read the outputs of `git worktree`.
  char w[GIT_WORKTREE_BUF_SZ];
  // Now, we fallback to `git worktree output`.
  waitpid(pid_worktree, NULL, 0);
  // By construction, wlen < GIT_WORKTREE_BUF_SZ;
  const int wlen = read(fd_worktree[0], w, GIT_WORKTREE_BUF_SZ - 1);
  debug_printf("Bytes read from `git worktree`: %d", wlen);
  w[wlen] = '\0';
  if (wlen == GIT_WORKTREE_BUF_SZ) {
    debug_printf("Warning: possibly missing data from `git worktree` due to\n"
                 "insufficient buffer size.%s",
                 "");
  }

  // We borrow argc to store the length of `GOAL`.
  argc = strlen(GOAL);
  for (c_right = w; (c_line = strsep(&c_right, "\n"));) {
    if (strncmp(c_line, "worktree ", 9) != 0) {
      continue;
    }
    c_line += 9;
    debug_printf("(worktree) %s", c_line);
    // Check to see if the current line ends with the goal.
    if ((c_left = c_right - argc - 1) < c_line) {
      continue;
    }
    if (strncmp(c_left, GOAL, argc) == 0) {
      // OUTPUT ==========
      write(STDOUT_FILENO, c_line, c_right - c_line - 1);
      return 64;
    }
  }

  debug_printf("Target not found. git-checkout2 is unable to help.%s", "");
  write(STDERR_FILENO, z, zlen);
  return exit_code;
}
