/* Independent Linux control for generic sparse-file and lifetime value laws.
 * Run in a disposable Linux 7.2.9 VM on noatime tmpfs, size=32K,nr_inodes=5.
 * No control object or C runtime is linked into the Topal kernel. */
#define _GNU_SOURCE
#include <errno.h>
#include <fcntl.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

#define CHECK(condition) do { if (!(condition)) { fprintf(stderr, "control:%d: %s errno=%d\n", __LINE__, #condition, errno); exit(1); } } while (0)
static const char *directory;
static int temporary(int flags) {
    int fd = open(directory, O_TMPFILE | flags, 0666);
    CHECK(fd >= 0);
    return fd;
}
static void seek_to(int fd, off_t position) { CHECK(lseek(fd, position, SEEK_SET) == position); }
static void write_all(int fd, const void *bytes, size_t length) {
    const unsigned char *cursor = bytes;
    while (length) { ssize_t count = write(fd, cursor, length); CHECK(count > 0); cursor += count; length -= (size_t)count; }
}
static void read_all(int fd, void *bytes, size_t length) {
    unsigned char *cursor = bytes;
    while (length) { ssize_t count = read(fd, cursor, length); CHECK(count > 0); cursor += count; length -= (size_t)count; }
}
static void close_file(int fd) { CHECK(close(fd) == 0); }
static struct stat metadata(int fd) { struct stat value; CHECK(fstat(fd, &value) == 0); return value; }

static void contents_and_sparse_extents(void) {
    int fd = temporary(O_RDWR), alias = dup(fd);
    CHECK(alias >= 0);
    struct stat initial = metadata(fd);
    CHECK(initial.st_size == 0 && initial.st_blocks == 0 && initial.st_nlink == 0);
    CHECK((initial.st_mode & 07777) == 0600 && S_ISREG(initial.st_mode));
    CHECK((fcntl(fd, F_GETFL) & O_ACCMODE) == O_RDWR);
    char value[16];
    CHECK(read(fd, value, sizeof(value)) == 0);
    write_all(fd, "abcdefgh", 8);
    seek_to(alias, 0); read_all(fd, value, 8); CHECK(memcmp(value, "abcdefgh", 8) == 0);
    CHECK(lseek(alias, 0, SEEK_CUR) == 8);
    CHECK(ftruncate(alias, 3) == 0 && lseek(fd, 0, SEEK_CUR) == 8);
    CHECK(ftruncate(fd, 4097) == 0);
    seek_to(fd, 0); memset(value, 0xa5, sizeof(value)); read_all(alias, value, 8);
    CHECK(memcmp(value, "abc\0\0\0\0\0", 8) == 0);
    CHECK(lseek(fd, 0, SEEK_DATA) == 0);
    CHECK(lseek(fd, 0, SEEK_HOLE) == 4096);
    errno = 0; CHECK(lseek(fd, 4096, SEEK_DATA) == -1 && errno == ENXIO);
    errno = 0; CHECK(lseek(fd, -1, SEEK_HOLE) == -1 && errno == ENXIO);
    CHECK(ftruncate(fd, 0) == 0 && metadata(fd).st_blocks == 0);
    const off_t large = ((off_t)1 << 48) + 13;
    seek_to(fd, large); write_all(alias, "sparse!!", 8);
    CHECK(metadata(fd).st_size == large + 8 && metadata(fd).st_blocks == 8);
    CHECK(lseek(fd, 0, SEEK_DATA) == ((off_t)1 << 48));
    CHECK(lseek(fd, large, SEEK_HOLE) == large + 8);
    close_file(fd); seek_to(alias, large); read_all(alias, value, 8);
    CHECK(memcmp(value, "sparse!!", 8) == 0); close_file(alias);
    fd = temporary(O_RDWR);
    const off_t maximum = INT64_MAX;
    seek_to(fd, maximum - 1);
    errno = 0; CHECK(write(fd, "two", 2) == -1 && errno == EINVAL);
    CHECK(write(fd, "t", 1) == 1);
    errno = 0; CHECK(write(fd, "x", 1) == -1 && errno == EINVAL);
    errno = 0; CHECK(lseek(fd, 1, SEEK_CUR) == -1 && errno == EINVAL);
    seek_to(fd, maximum - 1); memset(value, 0, sizeof(value));
    errno = 0; CHECK(read(fd, value, 8) == -1 && errno == EINVAL);
    CHECK(read(fd, value, 1) == 1 && value[0] == 't'); close_file(fd);
}

static void capacities_and_reuse(void) {
    int files[4];
    for (unsigned i = 0; i < 4; ++i) files[i] = temporary(O_RDWR);
    errno = 0; CHECK(open(directory, O_TMPFILE | O_RDWR, 0600) == -1 && errno == ENOSPC);
    for (unsigned i = 0; i < 4; ++i) {
        write_all(files[i], "first!!!", 8);
        seek_to(files[i], 4096); write_all(files[i], "second!!", 8);
    }
    seek_to(files[0], 8192); errno = 0;
    CHECK(write(files[0], "full", 4) == -1 && errno == ENOSPC);
    CHECK(lseek(files[0], 0, SEEK_CUR) == 8192 && metadata(files[0]).st_size == 4104);
    seek_to(files[0], 0); write_all(files[0], "override", 8);
    int alias = dup(files[1]); CHECK(alias >= 0); close_file(files[1]);
    errno = 0; CHECK(write(files[0], "samepage", 8) == 8);
    close_file(alias);
    files[1] = temporary(O_RDWR);
    CHECK(ftruncate(files[1], 8192) == 0 && metadata(files[1]).st_blocks == 0);
    char zero[16]; memset(zero, 0xa5, sizeof(zero)); read_all(files[1], zero, sizeof(zero));
    for (unsigned i = 0; i < sizeof(zero); ++i) CHECK(zero[i] == 0);
    for (unsigned i = 0; i < 4; ++i) close_file(files[i]);
}

static void append_and_fork(void) {
    int fd = temporary(O_RDWR | O_APPEND);
    write_all(fd, "parent!!", 8); seek_to(fd, 0);
    CHECK(write(fd, "", 0) == 0 && lseek(fd, 0, SEEK_CUR) == 0);
    pid_t child = fork(); CHECK(child >= 0);
    if (child == 0) {
        CHECK(umask(0777) == 0077); write_all(fd, "child!!!", 8); close_file(fd); _exit(11);
    }
    int status; CHECK(waitpid(child, &status, 0) == child && WIFEXITED(status) && WEXITSTATUS(status) == 11);
    CHECK(umask(0077) == 0077 && lseek(fd, 0, SEEK_CUR) == 16);
    seek_to(fd, 0); char value[16]; read_all(fd, value, sizeof(value));
    CHECK(memcmp(value, "parent!!child!!!", 16) == 0);
    CHECK(ftruncate(fd, INT64_MAX - 1) == 0); seek_to(fd, 0);
    CHECK(write(fd, "two", 2) == 1); seek_to(fd, 0);
    errno = 0; CHECK(write(fd, "x", 1) == -1 && errno == EFBIG);
    close_file(fd);
}

static void exec_child(int fd, int cloexec) {
    CHECK(fcntl(cloexec, F_GETFD) == -1 && errno == EBADF);
    CHECK(umask(0077) == 0077);
    CHECK(metadata(fd).st_size == 8);
    seek_to(fd, 0); char value[8]; read_all(fd, value, 8);
    CHECK(memcmp(value, "execdata", 8) == 0); close_file(fd);
}
static void exec_lifetime(void) {
    int fd = temporary(O_RDWR), cloexec = fcntl(fd, F_DUPFD_CLOEXEC, 14);
    CHECK(cloexec >= 14); write_all(fd, "execdata", 8);
    pid_t child = fork(); CHECK(child >= 0);
    if (child == 0) {
        char first[32], second[32]; snprintf(first, sizeof(first), "%d", fd); snprintf(second, sizeof(second), "%d", cloexec);
        execl("/proc/self/exe", "volatile-file-control", "--exec", first, second, (char *)NULL);
        _exit(91);
    }
    int status; CHECK(waitpid(child, &status, 0) == child && WIFEXITED(status) && WEXITSTATUS(status) == 0);
    close_file(cloexec); close_file(fd);
}

int main(int argc, char **argv) {
    if (argc == 4 && strcmp(argv[1], "--exec") == 0) { exec_child(atoi(argv[2]), atoi(argv[3])); return 0; }
    CHECK(argc == 2); directory = argv[1]; mode_t previous = umask(0077);
    contents_and_sparse_extents(); capacities_and_reuse(); append_and_fork(); exec_lifetime();
    CHECK(umask(previous) == 0077);
    puts("VOLATILE_FILE_CONTROL_PASS sparse aliases truncate regrowth page-budget object-budget append fork exec cloexec private-umask");
    return 0;
}
