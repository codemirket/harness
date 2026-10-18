// Linux ownership primitives for the container-only design runtime.
// No shell, credential handling or command rewriting occurs here.
#define _GNU_SOURCE
#include <sys/prctl.h>
#include <sys/file.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <signal.h>
#include <unistd.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>

static volatile sig_atomic_t stopped = 0;
static void stop(int sig) { stopped = sig; }
static void kill_children(void) {
  char path[128];
  snprintf(path, sizeof(path), "/proc/self/task/%ld/children", (long)getpid());
  FILE *f = fopen(path, "r");
  if (!f) { perror("runtime-guard children"); exit(125); }
  long pid;
  while (fscanf(f, "%ld", &pid) == 1) if (pid > 0) kill((pid_t)pid, SIGKILL);
  fclose(f);
}
int main(int argc, char **argv) {
  if (argc == 3 && strcmp(argv[1], "lock") == 0) {
    // Lock files remain as stable inodes. Never unlink a flock lock: a waiter
    // could otherwise hold an inode different from a newly arriving writer.
    int fd = open(argv[2], O_CREAT | O_RDWR | O_NOFOLLOW | O_CLOEXEC, 0600);
    struct stat st;
    if (fd < 0 || fstat(fd, &st) || !S_ISREG(st.st_mode) || st.st_nlink != 1) return 125;
    if (flock(fd, LOCK_EX | LOCK_NB)) { close(fd); return 73; }
    if (write(STDOUT_FILENO, "locked\n", 7) != 7) { close(fd); return 125; }
    char byte;
    while (read(STDIN_FILENO, &byte, 1) > 0) {}
    close(fd);
    return 0;
  }
  if (argc < 3 || strcmp(argv[1], "run") != 0) return 125;
  pid_t owner = getppid();
  if (owner == 1) return 125;
  if (prctl(PR_SET_CHILD_SUBREAPER, 1) || prctl(PR_SET_PDEATHSIG, SIGTERM)) return 125;
  if (getppid() != owner) return 125;
  struct sigaction sa = {0};
  sa.sa_handler = stop;
  sigemptyset(&sa.sa_mask);
  sigaction(SIGTERM, &sa, NULL);
  sigaction(SIGINT, &sa, NULL);
  sigaction(SIGHUP, &sa, NULL);
  pid_t child = fork();
  if (child < 0) return 125;
  if (!child) {
    signal(SIGTERM, SIG_DFL); signal(SIGINT, SIG_DFL); signal(SIGHUP, SIG_DFL);
    execvp(argv[2], argv + 2);
    perror("runtime-guard exec"); _exit(127);
  }
  int status = 0;
  while (!stopped) {
    pid_t result = waitpid(child, &status, 0);
    if (result == child) break;
    if (result < 0 && errno != EINTR) { stopped = SIGTERM; break; }
  }
  // Detached descendants are adopted by this subreaper. Killing each direct
  // child and reaping it adopts the next generation, including setsid children.
  for (;;) {
    kill_children();
    pid_t result;
    do { result = waitpid(-1, NULL, WNOHANG); } while (result > 0);
    if (result < 0 && errno == ECHILD) break;
    usleep(10000);
  }
  return stopped ? 128 + stopped : WIFEXITED(status) ? WEXITSTATUS(status) : 128 + WTERMSIG(status);
}
