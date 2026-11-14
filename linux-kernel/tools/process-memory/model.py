"""TK-DEC-284 / TOPAL-SYSTEMS-PROCESS-MM-001 portable heap transition model.

Objects and frame numbers describe reference state, never production authority.
No native execution, physical erasure, real TLB or asynchronous exclusion proof.
"""
from dataclasses import dataclass, replace


class InvalidState(ValueError):
    pass


def require(condition, reason):
    if not condition:
        raise InvalidState(reason)


@dataclass(frozen=True)
class Geometry:
    page: int = 4096
    start: int = 0x5F7000
    pages: int = 519
    frames: int = 2048

    def __post_init__(self):
        require(self.page > 0 and self.page & (self.page - 1) == 0, 'page')
        require(self.start > 0 and self.start % self.page == 0, 'start')
        require(self.pages > 0 and self.frames > 0, 'capacity')
        require(self.start + self.pages * self.page < 1 << 64, 'overflow')

    def count(self, program_break):
        return (program_break - self.start + self.page - 1) // self.page


@dataclass(frozen=True)
class Extent:
    token: int
    pid: int
    first: int
    count: int
    data: bytes

    @property
    def frames(self):
        return tuple(range(self.first, self.first + self.count))


@dataclass(frozen=True)
class Process:
    pid: int
    address_space: int
    generation: int
    program_break: int
    heap: int = 0
    return_value: int | None = None


@dataclass(frozen=True)
class Entry:
    pid: int
    address_space: int
    generation: int
    epoch: int


@dataclass(frozen=True)
class Loan:
    entry: Entry
    offset: int
    length: int
    writable: bool


@dataclass(frozen=True)
class Event:
    kind: str
    pid: int
    token: int = 0
    frames: tuple = ()
    data: bytes = b''


@dataclass(frozen=True)
class World:
    geometry: Geometry
    processes: tuple
    extents: tuple = ()
    active: int | None = 1
    child: int | None = None
    status: int | None = None
    waiting: bool = False
    next_pid: int = 2
    next_mm: int = 2
    next_token: int = 1
    next_epoch: int = 1
    entry: Entry | None = None
    loan: Loan | None = None

    def process(self, pid=None):
        matches = [p for p in self.processes if p.pid == (self.active if pid is None else pid)]
        require(len(matches) == 1, 'process owner')
        return matches[0]

    def extent(self, token):
        matches = [e for e in self.extents if e.token == token]
        require(len(matches) == 1, 'extent owner')
        return matches[0]

    def frames(self, pid=None):
        if self.active is None and pid is None:
            return ()
        p = self.process(pid)
        return self.extent(p.heap).frames if p.heap else ()


def initial(geometry=Geometry()):
    return World(geometry, (Process(1, 1, 0, geometry.start),))


def validate(w):
    g = w.geometry
    require(len(w.processes) <= 2, 'pair capacity')
    pids = [p.pid for p in w.processes]
    require(len(set(pids)) == len(pids), 'duplicate process')
    mm_ids = [p.address_space for p in w.processes]
    require(len(set(mm_ids)) == len(mm_ids) and all(0 < mm < w.next_mm for mm in mm_ids), 'MM identity')
    require((w.active in pids) if pids else w.active is None, 'active owner')
    require(not pids or 1 in pids, 'outer owner')
    require(all(0 < p < w.next_pid for p in pids), 'PID history')
    require(w.child is None or 1 < w.child < w.next_pid, 'child history')
    if w.child is None:
        require(pids in [[], [1]] and w.status is None and not w.waiting, 'no relationship')
    else:
        require((w.child in pids) == (w.status is None), 'child lifetime')
        require(set(pids) == ({1, w.child} if w.status is None else {1}), 'relationship owner')
    require(w.status is None or 0 <= w.status <= 255, 'status')
    require(not w.waiting or (w.child in pids and w.active == w.child), 'waiting owner')
    tokens = [e.token for e in w.extents]
    require(len(set(tokens)) == len(tokens), 'duplicate extent')
    require(all(0 < t < w.next_token for t in tokens), 'extent history')
    claimed = []
    frames = []
    for p in w.processes:
        require(p.generation >= 0, 'generation')
        require(g.start <= p.program_break <= g.start + g.page * g.pages, 'break')
        count = g.count(p.program_break)
        require(bool(p.heap) == bool(count), 'empty heap')
        if p.heap:
            e = w.extent(p.heap)
            require(e.pid == p.pid and e.count == count, 'heap owner/size')
            require(e.first >= 0 and e.first + count <= g.frames, 'physical domain')
            require(len(e.data) == count * g.page, 'backing size')
            claimed.append(e.token)
            frames.extend(e.frames)
    require(sorted(claimed) == sorted(tokens), 'unclaimed or aliased extent')
    require(len(set(frames)) == len(frames), 'physical alias')
    if w.entry is not None:
        p = w.process()
        require((w.entry.pid, w.entry.address_space, w.entry.generation) ==
                (p.pid, p.address_space, p.generation), 'entry owner')
        require(0 < w.entry.epoch < w.next_epoch, 'entry history')
    if w.loan is not None:
        require(w.entry is w.loan.entry, 'loan entry')
        require(w.loan.length >= 0 and w.loan.offset >= 0, 'loan range')
        if w.loan.length:
            e = w.extent(w.process().heap)
            require(w.loan.offset + w.loan.length <= len(e.data), 'loan backing')
    return w


def enter(w):
    validate(w)
    require(w.entry is None and w.loan is None, 'nested entry')
    p = w.process()
    entry = Entry(p.pid, p.address_space, p.generation, w.next_epoch)
    return replace(w, entry=entry, next_epoch=w.next_epoch + 1), entry


def admit(w, entry):
    validate(w)
    require(w.entry is entry and entry is not None, 'stale entry')
    require(w.loan is None, 'live copy loan')


def borrow(w, entry, address, length, writable=False):
    admit(w, entry)
    require(type(address) is int and type(length) is int, 'word types')
    require(0 <= address < 1 << 64 and 0 <= length < 1 << 64, 'word range')
    offset = 0
    if length:
        require(address + length < 1 << 64, 'range overflow')
        p = w.process()
        require(p.heap != 0, 'absent backing')
        offset = address - w.geometry.start
        require(0 <= offset and offset + length <= len(w.extent(p.heap).data), 'unmapped range')
    loan = Loan(entry, offset, length, writable)
    return replace(w, loan=loan), loan


def access(w, loan, data=None):
    validate(w)
    require(w.loan is loan and loan is not None and w.entry is loan.entry, 'stale loan')
    if data is not None:
        require(loan.writable and type(data) is bytes and len(data) == loan.length, 'write rights/size')
    if not loan.length:
        return w, b''
    e = w.extent(w.process().heap)
    result = e.data[loan.offset:loan.offset + loan.length]
    if data is not None:
        changed = replace(e, data=e.data[:loan.offset] + data + e.data[loan.offset + loan.length:])
        w = replace(w, extents=tuple(changed if x.token == e.token else x for x in w.extents))
    return w, result


def end_loan(w, loan):
    access(w, loan)
    return replace(w, loan=None)


def leave(w, entry):
    admit(w, entry)
    return replace(w, entry=None)


def update(w, p):
    return replace(w, processes=tuple(p if x.pid == p.pid else x for x in w.processes))


def allocate(w, pid, count, prefix=b''):
    if not count:
        return w, 0, ()
    occupied = {f for e in w.extents for f in e.frames}
    first = next((a for a in range(w.geometry.frames - count + 1)
                  if all(f not in occupied for f in range(a, a + count))), None)
    if first is None:
        return w, None, ()
    data = bytes(count * w.geometry.page)
    e = Extent(w.next_token, pid, first, count, prefix[:len(data)] + data[len(prefix):])
    events = (Event('allocate', pid, e.token, e.frames),
              Event('zero', pid, e.token, e.frames, data),
              Event('pin', pid, e.token, e.frames))
    return replace(w, extents=w.extents + (e,), next_token=w.next_token + 1), e.token, events


def publish(w, pid, token):
    old = w.frames()
    frames = w.extent(token).frames if token else ()
    return (Event('publish', pid, token, frames),
            Event('invalidate', pid, frames=tuple(range(max(len(old), len(frames))))))


def retire(w, token):
    if not token:
        return w, ()
    e = w.extent(token)
    events = (Event('unpin', e.pid, token, e.frames),
              Event('erase', e.pid, token, e.frames, bytes(len(e.data))),
              Event('release', e.pid, token, e.frames))
    return replace(w, extents=tuple(x for x in w.extents if x.token != token)), events


def finish(w, entry, value, events=()):
    require(w.entry is entry and w.loan is None, 'completion entry')
    return validate(replace(w, entry=None)), value, events


def brk(w, entry, request):
    admit(w, entry)
    p, g = w.process(), w.geometry
    require(type(request) is int and 0 <= request < 1 << 64, 'break word')
    if request == 0 or not g.start <= request <= g.start + g.page * g.pages:
        return finish(w, entry, p.program_break)
    count = g.count(request)
    old = w.extent(p.heap) if p.heap else None
    if count == (old.count if old else 0):
        return finish(update(w, replace(p, program_break=request)), entry, request)
    w1, token, events = allocate(w, p.pid, count, old.data if old else b'')
    if token is None:
        return finish(w, entry, p.program_break)
    events += publish(w1, p.pid, token)
    w1, ended = retire(w1, p.heap)
    w1 = update(w1, replace(p, program_break=request, heap=token))
    return finish(w1, entry, request, events + ended)


def fork(w, entry, preparation_unavailable=False):
    admit(w, entry)
    if w.child is not None:
        return finish(w, entry, -11)  # selected profile's occupied-slot EAGAIN
    p = w.process()
    require(p.pid == 1, 'creation owner')
    old = w.extent(p.heap) if p.heap else None
    w1, token, events = allocate(w, w.next_pid, old.count if old else 0, old.data if old else b'')
    if token is None:
        return finish(w, entry, -12)  # no memory for child; no PID committed
    if preparation_unavailable:
        # Model a later preparation miss, not an executed-kernel fault injection.
        # Issued extent history stays consumed even though no PID/MM is committed.
        w1, ended = retire(w1, token)
        return finish(w1, entry, -12, events + ended)
    child = Process(w.next_pid, w.next_mm, p.generation, p.program_break, token, 0)
    events += publish(w1, child.pid, token)
    w1 = update(w1, replace(p, return_value=child.pid))
    w1 = replace(w1, processes=w1.processes + (child,), active=child.pid,
                 child=child.pid, next_pid=child.pid + 1, next_mm=w.next_mm + 1)
    return finish(w1, entry, child.pid, events)


def handoff(w, entry):
    admit(w, entry)
    w1 = update(w, replace(w.process(), return_value=0))
    if len(w.processes) != 2 or w.waiting:
        return finish(w1, entry, 0)
    other = next(p for p in w.processes if p.pid != w.active)
    return finish(replace(w1, active=other.pid), entry, 0, publish(w, other.pid, other.heap))


def exec_reset(w, entry):
    admit(w, entry)
    p = w.process()
    events = publish(w, p.pid, 0)
    w, ended = retire(w, p.heap)
    w = update(w, replace(p, address_space=w.next_mm, generation=p.generation + 1,
                          program_break=w.geometry.start, heap=0))
    w = replace(w, next_mm=w.next_mm + 1)
    return finish(w, entry, 0, events + ended)


def child_exit(w, entry, status):
    admit(w, entry)
    require(w.active == w.child and type(status) is int, 'child exit owner')
    child, parent = w.process(), w.process(1)
    events = publish(w, parent.pid, parent.heap)
    w, ended = retire(w, child.heap)
    w = replace(w, processes=(parent,), active=1, status=status & 255, waiting=False)
    return finish(w, entry, 0, events + ended)


def wait(w, entry, output_valid=True, nohang=False):
    admit(w, entry)
    if w.active != 1 or w.child is None:
        return finish(w, entry, -10)  # ECHILD
    if w.status is None:
        if nohang:
            return finish(w, entry, 0)
        child = w.process(w.child)
        events = publish(w, child.pid, child.heap)
        return finish(replace(w, active=child.pid, waiting=True), entry, None, events)
    if not output_valid:
        return finish(w, entry, -14)  # EFAULT leaves the zombie for retry
    pid = w.child
    return finish(replace(w, child=None, status=None), entry, pid)


def terminate_outer(w, entry):
    admit(w, entry)
    require(w.active == 1, 'outer termination owner')
    events = publish(w, 1, 0)
    # Inactive child backing stays owned until this cancellation consumes it.
    for p in sorted(w.processes, key=lambda p: p.pid, reverse=True):
        w, ended = retire(w, p.heap)
        events += ended
    w = replace(w, processes=(), active=None, child=None, status=None, waiting=False)
    return finish(w, entry, 0, events)
