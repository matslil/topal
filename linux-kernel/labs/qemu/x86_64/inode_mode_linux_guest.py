"""Raw chmod-family oracle, executed only inside the disposable pinned VM."""
import ctypes
import errno
import json
import os
from pathlib import Path
import platform
import shutil
import stat
import subprocess
import tempfile

assert os.geteuid() == 0, 'prepared root credential required'
assert platform.release() == '7.2.9', platform.release()
libc = ctypes.CDLL(None, use_errno=True)
libc.syscall.restype = ctypes.c_long
events = []


def call(case, number, arguments, expected_errno=0):
    native = [ctypes.c_char_p(argument) if isinstance(argument, bytes)
              else ctypes.c_ulonglong(argument & ((1 << 64) - 1)) for argument in arguments]
    ctypes.set_errno(0)
    result = libc.syscall(ctypes.c_long(number), *native)
    observed = ctypes.get_errno() if result == -1 else 0
    assert (result, observed) == ((-1, expected_errno) if expected_errno else (0, 0)), (
        case, number, result, observed, expected_errno)
    events.append(dict(case=case, syscall=number, result=result, errno=observed))


def change(case, number, arguments, observation, permissions):
    before = observation()
    call(case, number, arguments)
    after = observation()
    assert stat.S_IFMT(after.st_mode) == stat.S_IFMT(before.st_mode), case
    assert stat.S_IMODE(after.st_mode) == permissions, (case, oct(after.st_mode), oct(permissions))
    assert (after.st_dev, after.st_ino, after.st_nlink, after.st_size,
            after.st_atime_ns, after.st_mtime_ns) == (
                before.st_dev, before.st_ino, before.st_nlink, before.st_size,
                before.st_atime_ns, before.st_mtime_ns), case
    assert after.st_ctime_ns >= before.st_ctime_ns, case
    events[-1]['mode_octal'] = oct(after.st_mode)
    events[-1]['preserved_type_identity_links_size_atime_mtime'] = True
    events[-1]['ctime_not_earlier'] = True


root = Path(tempfile.mkdtemp(prefix='topal-inode-linux-'))
descriptors = []
old_cwd = os.open('.', os.O_RDONLY | os.O_DIRECTORY)
readonly = root / 'readonly'
mounted = False
try:
    filename = root / 'file'
    encoded = os.fsencode(filename)
    writer = os.open(filename, os.O_CREAT | os.O_EXCL | os.O_RDWR, 0o644)
    descriptors.append(writer)
    os.write(writer, b'payload')
    reader = os.open(filename, os.O_RDONLY)
    descriptors.append(reader)
    alias = os.dup(reader)
    descriptors.append(alias)
    directory = os.open(root, os.O_RDONLY | os.O_DIRECTORY)
    descriptors.append(directory)
    observe = lambda: os.fstat(reader)
    original_umask = os.umask(0o777)
    try:
        change('chmod-high-native-mode-and-file-type-bits', 90,
               [encoded, 0xFEDCBA9876548ED1], observe, 0o7321)
        change('fchmod-readonly-description', 91, [reader, 0o6754], observe, 0o6754)
        change('fchmod-native-unsigned-fd-cast', 91,
               [reader | (1 << 32), 0o4755], observe, 0o4755)
        change('fchmodat-ignores-fourth-argument', 268,
               [directory, b'file', 0o1234, (1 << 64) - 1], observe, 0o1234)
        change('fchmodat2-absolute-ignores-bad-dirfd', 452,
               [-1, encoded, (1 << 64) - 1, 0], observe, 0o7777)
        change('fchmodat2-native-unsigned-flags-cast', 452,
               [directory, b'file', 0o600, 1 << 32], observe, 0o600)
        change('fchmodat2-nofollow-existing-nonsymlink', 452,
               [directory, b'file', 0o640, 0x100], observe, 0o640)
        change('fchmodat2-empty-readonly-regular', 452,
               [reader, b'', 0o601, 0x1000], observe, 0o601)
        change('fchmodat2-empty-readonly-alias', 452,
               [alias, b'', 0o602, 0x1100], observe, 0o602)
        change('fchmod-equal-mode-still-valid', 91, [reader, 0o602], observe, 0o602)
        change('fchmod-existing-description-retains-write-grant', 91,
               [reader, 0], observe, 0)
        assert os.pwrite(writer, b'!', 0) == 1
        try:
            os.write(reader, b'forbidden')
            raise AssertionError('chmod widened descriptor access rights')
        except OSError as error:
            assert error.errno == errno.EBADF
    finally:
        os.umask(original_umask)

    call('fchmod-bad-fd', 91, [-1, 0o777], errno.EBADF)
    call('fchmodat2-flags-before-copy-and-fd', 452, [-1, 0, 0o777, 0x200], errno.EINVAL)
    call('fchmodat2-null-empty-flag-before-bad-fd', 452,
         [-1, 0, 0o777, 0x1000], errno.EFAULT)
    call('fchmodat2-null-empty-flag-valid-fd', 452,
         [reader, 0, 0o777, 0x1000], errno.EFAULT)
    call('fchmodat2-null-without-empty-flag', 452, [reader, 0, 0o777, 0], errno.EFAULT)
    call('fchmodat2-empty-without-flag-before-bad-fd', 452,
         [-1, b'', 0o777, 0], errno.ENOENT)
    call('fchmodat2-empty-bad-fd', 452, [-1, b'', 0o777, 0x1000], errno.EBADF)
    call('fchmodat2-relative-bad-fd', 452, [-1, b'file', 0o777, 0], errno.EBADF)
    call('fchmodat2-relative-nondirectory-fd', 452,
         [reader, b'file', 0o777, 0], errno.ENOTDIR)
    call('chmod-regular-trailing-slash', 90, [encoded + b'/', 0o777], errno.ENOTDIR)
    call('chmod-missing', 90, [os.fsencode(root / 'missing'), 0o777], errno.ENOENT)
    os.chdir(root)
    change('fchmodat-relative-cwd', 268, [-100, b'file', 0o640], observe, 0o640)
    change('fchmodat2-empty-cwd', 452, [-100, b'', 0o2750, 0x1000],
           lambda: os.fstat(directory), 0o2750)
    change('fchmod-readonly-directory', 91, [directory, 0o1777],
           lambda: os.fstat(directory), 0o1777)
    os.fchdir(old_cwd)

    original_identity = observe().st_ino
    os.unlink(filename)
    fresh = os.open(filename, os.O_CREAT | os.O_EXCL | os.O_RDWR, 0o600)
    descriptors.append(fresh)
    assert os.fstat(fresh).st_ino != original_identity
    before_fresh = os.fstat(fresh)
    change('fchmod-retained-unlinked-inode', 91, [reader, 0o4701], observe, 0o4701)
    assert os.fstat(fresh).st_mode == before_fresh.st_mode
    assert os.fstat(fresh).st_ctime_ns == before_fresh.st_ctime_ns
    change('chmod-recreated-name-distinct-from-unlinked', 90,
           [encoded, 0o4222], lambda: os.fstat(fresh), 0o4222)
    assert stat.S_IMODE(observe().st_mode) == 0o4701

    pipe_read, pipe_write = os.pipe()
    descriptors.extend([pipe_read, pipe_write])
    change('fchmod-pipe-read-end', 91, [pipe_read, 0o6754],
           lambda: os.fstat(pipe_read), 0o6754)
    assert stat.S_IMODE(os.fstat(pipe_write).st_mode) == 0o6754
    change('fchmodat2-empty-pipe-write-end', 452,
           [pipe_write, b'', 0o1234, 0x1000], lambda: os.fstat(pipe_write), 0o1234)
    assert stat.S_IMODE(os.fstat(pipe_read).st_mode) == 0o1234
    assert os.write(pipe_write, b'pipe') == 4
    assert os.read(pipe_read, 4) == b'pipe'

    readonly.mkdir()
    subprocess.run(['mount', '-t', 'tmpfs', '-o', 'size=64k', 'tmpfs', str(readonly)], check=True)
    mounted = True
    target = readonly / 'file'
    target.write_bytes(b'immutable mount control')
    read_fd = os.open(target, os.O_RDONLY)
    descriptors.append(read_fd)
    subprocess.run(['mount', '-o', 'remount,ro', str(readonly)], check=True)
    call('chmod-readonly-existing-target', 90, [os.fsencode(target), 0o777], errno.EROFS)
    call('fchmod-readonly-existing-target', 91, [read_fd, 0o777], errno.EROFS)
    call('chmod-missing-target-before-readonly-mount-denial', 90,
         [os.fsencode(readonly / 'missing'), 0o777], errno.ENOENT)
    call('fchmodat2-empty-readonly-existing-target', 452,
         [read_fd, b'', 0o777, 0x1000], errno.EROFS)
finally:
    os.fchdir(old_cwd)
    os.close(old_cwd)
    for descriptor in descriptors:
        os.close(descriptor)
    if mounted:
        subprocess.run(['umount', str(readonly)], check=True)
    shutil.rmtree(root)

print(json.dumps(dict(schema='topal-linux-inode-mode-control/1',
    kernel_release=platform.release(), effective_uid=os.geteuid(),
    events=events, passed=True), sort_keys=True))
