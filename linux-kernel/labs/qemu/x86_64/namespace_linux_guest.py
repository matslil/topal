"""Raw namespace laws/ordering, run only in the disposable pinned Linux VM."""
import ctypes
import errno
import fcntl
import json
import os
from pathlib import Path
import platform
import shutil
import stat
import subprocess
import tempfile

assert os.environ.get('TOPAL_NAMESPACE_LINUX_GUEST') == '1'
assert os.geteuid() == 0 and platform.machine() == 'x86_64'
assert platform.release() == '7.2.9', platform.release()
libc = ctypes.CDLL(None, use_errno=True)
libc.syscall.restype = ctypes.c_long
events = []
fds = set()
root = Path(tempfile.mkdtemp(prefix='topal-namespace-linux-'))
mutable, readonly = root / 'mutable', root / 'readonly'
mounted = []
original_cwd = os.open('.', os.O_RDONLY | os.O_DIRECTORY)


def opened(path, flags=os.O_RDONLY, mode=0o640):
    fd = os.open(path, flags, mode)
    fds.add(fd)
    return fd


def close(fd):
    os.close(fd)
    fds.remove(fd)


def raw(number, arguments):
    native = [ctypes.c_char_p(arg) if isinstance(arg, bytes)
              else ctypes.c_ulonglong(arg & ((1 << 64) - 1)) for arg in arguments]
    ctypes.set_errno(0)
    value = libc.syscall(ctypes.c_long(number), *native)
    return value, ctypes.get_errno() if value == -1 else 0


def call(case, number, arguments, expected_errno=0):
    observed = raw(number, arguments)
    expected = (-1, expected_errno) if expected_errno else (0, 0)
    assert observed == expected, (case, number, observed, expected)
    events.append(dict(case=case, syscall=number, result=observed[0], errno=observed[1],
                       asserted_errno=True))


def inode(st):
    return (st.st_dev, st.st_ino, st.st_nlink, st.st_mode, st.st_uid, st.st_gid,
            st.st_size, st.st_blocks, st.st_atime_ns, st.st_mtime_ns, st.st_ctime_ns)


def state():
    result = {}
    for label, directory in [('mutable', mutable), ('readonly', readonly)]:
        result[label] = inode(os.stat(directory))
        for name in sorted(os.listdir(os.fsencode(directory))):
            path = os.fsencode(directory) + b'/' + name
            result[label + '/' + name.hex()] = (inode(os.stat(path)), Path(os.fsdecode(path)).read_bytes())
    for fd in sorted(fds):
        st = os.fstat(fd)
        position = os.lseek(fd, 0, os.SEEK_CUR)
        data = None
        if stat.S_ISREG(st.st_mode) and fcntl.fcntl(fd, fcntl.F_GETFL) & os.O_ACCMODE != os.O_WRONLY:
            data = os.pread(fd, st.st_size, 0)
        result['fd/' + str(fd)] = (inode(st), position, data)
    return result


def refuse(case, number, arguments, expected_errno):
    before = state()
    observed = raw(number, arguments)
    assert observed[0] == -1 and observed[1] > 0, (case, observed)
    assert observed[1] == expected_errno, (case, observed, expected_errno)
    assert state() == before, (case, 'refusal changed namespace, inode, bytes or descriptor state')
    events.append(dict(case=case, syscall=number, result=observed[0], errno=observed[1],
                       asserted_errno=True, complete_state_preserved=True))


def preserved_inode(before, after, links):
    assert (after.st_dev, after.st_ino, after.st_mode, after.st_uid, after.st_gid,
            after.st_size, after.st_blocks, after.st_atime_ns, after.st_mtime_ns) == (
                before.st_dev, before.st_ino, before.st_mode, before.st_uid, before.st_gid,
                before.st_size, before.st_blocks, before.st_atime_ns, before.st_mtime_ns)
    assert after.st_nlink == links and after.st_ctime_ns >= before.st_ctime_ns


def reset_files():
    for fd in list(fds):
        if fd not in directory_fds:
            close(fd)
    for name in os.listdir(os.fsencode(mutable)):
        os.unlink(os.fsencode(mutable) + b'/' + name)


def create(name, data=b'payload'):
    fd = opened(os.fsencode(mutable) + b'/' + name, os.O_CREAT | os.O_EXCL | os.O_RDWR)
    assert os.write(fd, data) == len(data)
    return fd


try:
    for directory in [mutable, readonly]:
        directory.mkdir()
        subprocess.run(['mount', '-t', 'tmpfs', '-o', 'size=64k,nr_inodes=128,mode=1777,noatime',
                        'topal-namespace-control', str(directory)], check=True)
        mounted.append(directory)
    (readonly / 'exists').write_bytes(b'immutable mount control')
    subprocess.run(['mount', '-o', 'remount,ro', str(readonly)], check=True)
    directory = opened(mutable, os.O_RDONLY | os.O_DIRECTORY)
    read_directory = opened(readonly, os.O_RDONLY | os.O_DIRECTORY)
    directory_fds = {directory, read_directory}
    os.chdir(mutable)

    # All five raw adapters, with access/position/identity independent of names.
    writer = create(b'a')
    reader = opened(b'a')
    other = opened(b'a')
    assert os.lseek(other, 2, os.SEEK_SET) == 2
    initial = os.fstat(writer)
    parent_before = os.stat(mutable)
    call('link-shared-inode', 86, [b'a', b'alias'])
    preserved_inode(initial, os.fstat(writer), 2)
    assert os.stat(mutable).st_mtime_ns >= parent_before.st_mtime_ns
    assert os.stat(mutable).st_ctime_ns >= parent_before.st_ctime_ns
    assert os.stat(b'a').st_ino == os.stat(b'alias').st_ino == initial.st_ino
    assert os.fstat(reader).st_nlink == 2
    assert os.pread(reader, 7, 0) == b'payload'
    assert (os.lseek(writer, 0, os.SEEK_CUR), os.lseek(reader, 0, os.SEEK_CUR),
            os.lseek(other, 0, os.SEEK_CUR)) == (7, 0, 2)
    call('rename-moves-name', 82, [b'a', b'moved'])
    preserved_inode(initial, os.fstat(writer), 2)
    assert not os.path.exists(b'a') and os.stat(b'moved').st_ino == initial.st_ino
    call('renameat-native-dirfd-casts', 264,
         [directory | (1 << 32), b'moved', directory | (1 << 32), b'at'])
    call('linkat-native-dirfd-and-flags-casts', 265,
         [directory | (1 << 32), b'at', directory | (1 << 32), b'follow', (1 << 32) | 0x400])
    assert os.fstat(writer).st_nlink == 3
    refuse('hard-link-does-not-widen-readonly-description', 1,
           [reader, b'x', 1], errno.EBADF)
    before = state()
    for case, number, arguments in [
        ('rename-same-inode-alias', 82, [b'at', b'alias']),
        ('renameat2-same-inode-exchange', 316, [directory, b'at', directory, b'alias', 2]),
        ('rename-identical-path', 82, [b'at', b'at']),
    ]:
        call(case, number, arguments)
        assert state() == before, case
    refuse('noreplace-before-same-inode-noop', 316,
           [directory, b'at', directory, b'alias', 1], errno.EEXIST)
    call('renameat2-noreplace-native-flags-cast', 316,
         [directory, b'at', directory, b'new', (1 << 32) | 1])
    assert os.stat(b'new').st_ino == initial.st_ino
    assert (os.lseek(writer, 0, os.SEEK_CUR), os.lseek(other, 0, os.SEEK_CUR)) == (7, 2)
    reset_files()

    # Replacement victim lifetime and distinct identity while its fd remains.
    source, victim = create(b'source', b'source bytes'), create(b'victim', b'victim bytes')
    source_before, victim_before = os.fstat(source), os.fstat(victim)
    old_victim = victim_before.st_ino
    call('rename-replacement-retains-open-victim', 82, [b'source', b'victim'])
    preserved_inode(source_before, os.fstat(source), 1)
    preserved_inode(victim_before, os.fstat(victim), 0)
    assert os.fstat(victim).st_nlink == 0 and os.pread(victim, 12, 0) == b'victim bytes'
    assert os.stat(b'victim').st_ino == os.fstat(source).st_ino != old_victim
    os.unlink(b'victim')
    fresh = create(b'victim', b'fresh')
    assert os.fstat(fresh).st_ino not in [old_victim, os.fstat(source).st_ino]
    refuse('unlinked-once-linked-fd-cannot-resurrect', 265,
           [victim, b'', directory, b'resurrect', 0x1000], errno.ENOENT)
    reset_files()

    # Unequal link counts, full name budget, and exchange preserve associations.
    left, right = create(b'left', b'left bytes'), create(b'right', b'right bytes')
    call('link-before-exchange', 86, [b'left', b'left-alias'])
    create(b'fourth', b'fourth')
    old_left, old_right = os.fstat(left), os.fstat(right)
    call('renameat2-exchange-unequal-link-counts', 316,
         [directory, b'left', directory, b'right', 2])
    assert os.stat(b'left').st_ino == old_right.st_ino
    assert os.stat(b'right').st_ino == old_left.st_ino
    assert (os.fstat(left).st_nlink, os.fstat(right).st_nlink) == (2, 1)
    preserved_inode(old_left, os.fstat(left), 2)
    preserved_inode(old_right, os.fstat(right), 1)
    call('rename-full-entry-budget-to-absent', 82, [b'fourth', b'new-fourth'])
    assert set(os.listdir(mutable)) == {'left', 'right', 'left-alias', 'new-fourth'}
    reset_files()

    # Empty-path initial linking consumes eligibility; readonly descriptions
    # retain their own access rights. O_TMPFILE|O_EXCL is never eligible.
    temporary = opened(mutable, os.O_TMPFILE | os.O_RDWR)
    assert os.write(temporary, b'temporary') == 9
    readonly_temporary = opened('/proc/self/fd/' + str(temporary))
    call('linkat-initial-tmpfile-readonly-description', 265,
         [readonly_temporary, b'', directory, b'first-link', 0x1000])
    call('linkat-already-linked-temporary-description', 265,
         [readonly_temporary, b'', directory, b'second-link', 0x1000])
    assert os.fstat(temporary).st_nlink == 2 and os.pread(readonly_temporary, 9, 0) == b'temporary'
    os.unlink(b'first-link'); os.unlink(b'second-link')
    refuse('initial-link-eligibility-not-recreated-after-unlink', 265,
           [temporary, b'', directory, b'no-resurrection', 0x1000], errno.ENOENT)
    exclusive = opened(mutable, os.O_TMPFILE | os.O_EXCL | os.O_RDWR)
    refuse('exclusive-anonymous-is-ineligible', 265,
           [exclusive, b'', directory, b'no-exclusive-link', 0x1000], errno.ENOENT)
    reset_files()

    raw_name = bytes(128 + i % 128 for i in range(255))
    create(b'raw')
    call('rename-long-raw-component', 82, [b'raw', raw_name])
    assert os.listdir(os.fsencode(mutable)) == [raw_name]
    call('linkat-empty-descriptor-to-raw-name', 265,
         [directory, raw_name, directory, b'raw-alias', 0])
    assert os.stat(raw_name).st_ino == os.stat(b'raw-alias').st_ino
    reset_files()

    # Primary-source ordering matrix; each refusal additionally proves whole
    # namespace/inode/data/description preservation. Expectations are explicit
    # and were cross-checked against pinned source and an earlier VM observation.
    file = create(b'file')
    create(b'exists')
    ro = os.fsencode(readonly)
    for case, number, arguments, error in [
        ('rename-flags-before-bad-names', 316, [-1, 0, -1, 0, 8], errno.EINVAL),
        ('rename-exchange-noreplace-invalid', 316, [-1, 0, -1, 0, 3], errno.EINVAL),
        ('rename-exchange-whiteout-invalid', 316, [-1, 0, -1, 0, 6], errno.EINVAL),
        ('link-flags-before-bad-names', 265, [-1, 0, -1, 0, 1], errno.EINVAL),
        ('rename-old-null', 82, [0, b'missing'], errno.EFAULT),
        ('rename-new-null', 82, [b'file', 0], errno.EFAULT),
        ('link-null-with-empty-flag', 265, [file, 0, directory, b'new', 0x1000], errno.EFAULT),
        ('link-empty-without-flag-before-bad-fd', 265, [-1, b'', directory, b'new', 0], errno.ENOENT),
        ('link-empty-bad-fd', 265, [-1, b'', directory, b'new', 0x1000], errno.EBADF),
        ('rename-bad-old-dirfd', 264, [-1, b'file', directory, b'new'], errno.EBADF),
        ('rename-bad-new-dirfd', 264, [directory, b'file', -1, b'new'], errno.EBADF),
        ('link-relative-nondirectory', 265, [file, b'file', directory, b'new', 0], errno.ENOTDIR),
        ('rename-old-parent-missing-before-new-copy', 82, [b'missing-parent/file', 0], errno.ENOENT),
        ('rename-old-final-missing-before-new-copy', 82, [b'missing', 0], errno.EFAULT),
        ('link-old-missing-before-new-copy', 86, [b'missing', 0], errno.ENOENT),
        ('rename-old-missing-new-parent-missing', 82, [b'missing', b'missing-parent/new'], errno.ENOENT),
        ('rename-missing-source-existing-noreplace', 316, [directory, b'missing', directory, b'exists', 1], errno.ENOENT),
        ('rename-missing-source-cross-mount', 82, [b'missing', ro + b'/new'], errno.EXDEV),
        ('rename-existing-source-cross-mount-existing-noreplace', 316, [directory, b'file', read_directory, b'exists', 1], errno.EXDEV),
        ('rename-readonly-missing-source', 82, [ro + b'/missing', ro + b'/new'], errno.EROFS),
        ('rename-readonly-existing-noreplace', 316, [read_directory, b'exists', read_directory, b'exists', 1], errno.EROFS),
        ('rename-old-dot-cross-mount', 82, [b'.', ro + b'/new'], errno.EXDEV),
        ('rename-old-dot-same-mount', 82, [b'.', b'new'], errno.EBUSY),
        ('rename-new-dot-noreplace', 316, [directory, b'file', directory, b'.', 1], errno.EEXIST),
        ('rename-new-dot-plain', 82, [b'file', b'.'], errno.EBUSY),
        ('rename-existing-old-trailing-cross-mount', 82, [b'file/', ro + b'/new'], errno.EXDEV),
        ('rename-existing-old-trailing', 82, [b'file/', b'new'], errno.ENOTDIR),
        ('rename-missing-old-trailing', 82, [b'missing/', b'new'], errno.ENOENT),
        ('rename-new-trailing-existing-regular', 82, [b'file', b'exists/'], errno.ENOTDIR),
        ('rename-new-trailing-missing', 82, [b'file', b'new/'], errno.ENOTDIR),
        ('link-missing-old-before-readonly-new', 86, [b'missing', ro + b'/new'], errno.ENOENT),
        ('link-cross-mount-new-missing-readonly', 86, [b'file', ro + b'/new'], errno.EROFS),
        ('link-cross-mount-new-existing-readonly', 86, [b'file', ro + b'/exists'], errno.EEXIST),
        ('link-readonly-old-to-mutable-missing', 86, [ro + b'/exists', b'new'], errno.EXDEV),
        ('link-readonly-old-to-mutable-existing', 86, [ro + b'/exists', b'exists'], errno.EEXIST),
        ('link-new-dot', 86, [b'file', b'.'], errno.EEXIST),
        ('link-new-trailing-missing', 86, [b'file', b'new/'], errno.ENOENT),
        ('link-new-trailing-existing', 86, [b'file', b'exists/'], errno.EEXIST),
        ('link-old-regular-trailing', 86, [b'file/', b'new'], errno.ENOTDIR),
        ('rename-long-final-cross-mount', 82, [b'x' * 256, ro + b'/new'], errno.EXDEV),
        ('rename-readonly-long-final', 82, [ro + b'/' + b'x' * 256, ro + b'/new'], errno.EROFS),
        ('link-long-old-before-readonly-new', 86, [b'x' * 256, ro + b'/new'], errno.ENAMETOOLONG),
        ('link-readonly-new-long-final', 86, [b'file', ro + b'/' + b'x' * 256], errno.ENAMETOOLONG),
        ('rename-exchange-missing-target', 316, [directory, b'file', directory, b'new', 2], errno.ENOENT),
        ('renameat-bad-old-fd-ignored-for-absolute-new-fault', 264, [-1, os.fsencode(mutable / 'file'), -1, 0], errno.EFAULT),
    ]:
        refuse(case, number, arguments, error)
finally:
    os.fchdir(original_cwd)
    os.close(original_cwd)
    for fd in list(fds):
        close(fd)
    for directory in reversed(mounted):
        subprocess.run(['umount', str(directory)], check=True)
    shutil.rmtree(root)

print(json.dumps(dict(schema='topal-linux-namespace-control/1', kernel_release=platform.release(),
    effective_uid=os.geteuid(), events=events, passed=True,
    scope='raw pinned Linux errno baseline, metadata/lifetime laws and complete refusal-state preservation; not Topal handler qualification'), sort_keys=True))
