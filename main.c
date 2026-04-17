// STDOUT is reserved for printing the target directory. Always try to return
// the exit code of the underlying `git checkout` command.

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

#define GIT_CHECKOUT_BUF_MAX 512
#define GIT_WORKTREE_BUF_MAX 2048

// Ban list.
// 1. printf: in production we want zero allocations.
#define BANNED(func) sorry__##func##__is_a_banned_function
#undef printf
#define printf(...) BANNED(strcpy)

#define STARTS_WITH(haystack, prefix)                                          \
  (strncmp(haystack, prefix, sizeof(prefix) - 1) == 0)

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

struct pipedata {
  int fd[2];
  pid_t pid;
};

#define PIPE_AND_FORK(pd, COMMAND)                                             \
  if (pipe(pd.fd) == -1) {                                                     \
    ERR("pipe failed. This is necessary to run `git " #COMMAND "`.");          \
    return 1;                                                                  \
  } else if ((pd.pid = fork()) == -1) {                                        \
    ERR("fork failed. This is necessary to run `git " #COMMAND "`.");          \
    return 1;                                                                  \
  }

// Returns 64 if this function's stdout output is meant to be taken as the
// target directory for the `cd` command.
int main(int argc, char *argv[]) {
  debug_printf("\x1b[33mDEBUG MODE\x1b[m", 0);

  setup_git_binary();
  debug_printf("GIT = %s", GIT);

  // If the number of arguments (excluding the command) is not 1, then we revert
  // to original `git checkout` behaviour. This ensures that there is simplicity
  // in the code following this.
  if (argc != 2) {
    char *argv2[argc + 2]; // +1 for "checkout", +1 for NULL.
    argv2[0] = GIT;
    argv2[1] = "checkout";
    memcpy(argv2[2], argv, argc - 1);
    argv[argc + 2] = NULL;
    execvp(GIT, argv2);
  }
  debug_printf("Indeed only has one CLI argument.", 0);

  // Since there's only one CLI argument, we shall call it GOAL.
#define GOAL argv[1]

  // Pipe data for `git checkout`.
  struct pipedata pd_c;
  // Pipe data for `git worktree`.
  struct pipedata pd_w;

  PIPE_AND_FORK(pd_c, checkout);

  /* Child process: `git checkout` */
  if (pd_c.pid == 0) {
    // Capture `git checkout` STDERR.
    dup2(pd_c.fd[1], STDERR_FILENO);
    // Don't send anything to STDOUT. Remember, STDOUT is reserved for printing
    // the GOAL's directory.
    close(STDOUT_FILENO);
    close(pd_c.fd[0]), close(pd_c.fd[1]);
    execlp(GIT, GIT, "checkout", GOAL, NULL);
  } else {
    close(pd_c.fd[1]);
  }

  PIPE_AND_FORK(pd_w, worktree);

  /* Child process: `git worktree` */
  if (pd_w.pid == 0) { //
    // Capture `git worktree` STDOUT.
    dup2(pd_w.fd[1], STDOUT_FILENO);
    // Don't listen to stderr. We shall not parse that at all.
    close(STDERR_FILENO);
    close(pd_w.fd[0]), close(pd_w.fd[1]);
    execlp(GIT, GIT, "worktree", "list", "--porcelain", NULL);
  } else {
    close(pd_w.fd[1]);
  }

  int git_checkout_exit_code;
  waitpid(pd_c.pid, &git_checkout_exit_code, 0);
  git_checkout_exit_code >>= 8; // Get the high bits.

  /// `git checkout` byte buffer.
  char buf_c[GIT_CHECKOUT_BUF_MAX];

  // By construction, buf_c_len < GIT_CHECKOUT_BUF_MAX.
  const int buf_c_len = read(pd_c.fd[0], buf_c, GIT_CHECKOUT_BUF_MAX - 1);
  debug_printf("Bytes read from `git checkout`: %d", buf_c_len);
  buf_c[buf_c_len] = '\0';
  if (buf_c_len + 3 == GIT_CHECKOUT_BUF_MAX) {
    ERR("Warning: possibly missing bytes from `git checkout` output");
    ERR("due to insufficient buffer size.");
  }

  debug_printf("GIT CHECKOUT OUTPUT: \"%s\"", buf_c);

  // If all goes well, just reflect `git checkout's` stderr output back to the
  // terminal's stderr.
  if (git_checkout_exit_code == 0) {
    write(STDERR_FILENO, buf_c, buf_c_len);
    debug_printf("Everything went well, nothing to do anymore.", 0);
    return git_checkout_exit_code;
  }

  // If it turns out we're not even in a git repository, then exit early.
  if (STARTS_WITH(buf_c, "fatal: not a git repository")) {
    write(STDERR_FILENO, buf_c, buf_c_len);
    debug_printf("Not in a git repo, exit code %d", git_checkout_exit_code);
    return git_checkout_exit_code;
  }

  // `git checkout` sometimes would override local changes, in which case the
  // error message is:
  // ---------------------------------------------------------------------------
  // error: Your local changes to the following files would be overwritten by
  // checkout:
  //   ...
  // Please commit your changes or stash them before you switch branches.
  // Aborting
  if (strncmp(buf_c, "error: Your local changes t", 27) == 0) {
    write(STDERR_FILENO, buf_c, buf_c_len);
    debug_printf("Your local changes (exit code: %d)", git_checkout_exit_code);
    return git_checkout_exit_code;
  }

  char *c_left, *c_right, *c_line;

  // Now this is where `git-checkout2` shines.
  //
  // If we see that this branch is already used at another worktree, we print
  // the directory to that worktree to STDOUT so the parent shell can go there.
  if (STARTS_WITH(buf_c, "fatal:") &&
      ((c_left = strstr(buf_c, "is already used by worktree at")) != NULL)) {
    if ((c_left = strchr(c_left + 30, '\'')) == NULL) {
      ERR("Parsing error at \"is already used by worktree at...\"");
    }
    if ((c_right = strchr(++c_left, '\'')) == NULL) {
      ERR("Parsing error at \"is already used by worktree at...\"");
    }
    // OUTPUT ==========
    write(STDOUT_FILENO, c_left, c_right - c_left);
    return 64;
  }

  // Now, we fallback to `git worktree output`.
  waitpid(pd_w.pid, NULL, 0);

  /// `git worktree` byte buffer.
  char buf_w[GIT_WORKTREE_BUF_MAX];

  // By construction, buf_w_len < GIT_WORKTREE_BUF_MAX.
  const int buf_w_len = read(pd_w.fd[0], buf_w, GIT_WORKTREE_BUF_MAX - 1);
  debug_printf("Bytes read from `git worktree`: %d", buf_w_len);
  buf_w[buf_w_len] = '\0';

  if (buf_w_len + 3 == GIT_WORKTREE_BUF_MAX) {
    ERR("Warning: possibly missing bytes from `git worktree` output");
    ERR("due to insufficient buffer size.");
  }

  // We borrow argc to store the length of `GOAL`.
  argc = strlen(GOAL);

  for (c_right = buf_w; (c_line = strsep(&c_right, "\n"));) {
    if (!STARTS_WITH(c_line, "worktree ")) {
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

  debug_printf("Target not found. git-checkout2 is unable to help.", 0);
  write(STDERR_FILENO, buf_c, buf_c_len);
  return git_checkout_exit_code;
}
