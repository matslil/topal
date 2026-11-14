"""Separate ordering checker for saved reference-model events, not native logs."""
from model import require, validate


def check(before, after, events):
    validate(before)
    validate(after)
    held = {e.token: (e.pid, e.frames) for e in before.extents}
    pinned = set(held)
    zeroed, erased = set(), set()
    visible = before.frames()
    cached = visible
    selected = before.active
    pending = None
    next_token = before.next_token
    for e in events:
        if e.kind == 'allocate':
            require(e.token == next_token and e.token not in held, 'issuance')
            next_token += 1
            occupied = {f for _, fs in held.values() for f in fs}
            require(e.frames and len(set(e.frames)) == len(e.frames), 'allocation shape')
            require(tuple(range(e.frames[0], e.frames[0] + len(e.frames))) == e.frames, 'contiguous extent')
            require(all(0 <= f < before.geometry.frames and f not in occupied for f in e.frames), 'allocation alias')
            held[e.token] = (e.pid, e.frames)
        elif e.kind in ['zero', 'pin', 'unpin', 'erase', 'release']:
            require(held.get(e.token) == (e.pid, e.frames), 'event owner')
            if e.kind in ['zero', 'erase']:
                require(e.token not in pinned, 'zero/erase pinned extent')
                require(len(e.data) == len(e.frames) * before.geometry.page and not any(e.data), 'complete erasure')
                (zeroed if e.kind == 'zero' else erased).add(e.token)
            elif e.kind == 'pin':
                require(e.token in zeroed and e.token not in pinned, 'pin before zero/duplicate pin')
                pinned.add(e.token)
            elif e.kind == 'unpin':
                require(e.token in pinned, 'missing pin')
                require(not set(e.frames).intersection(visible + cached), 'unpin before revocation/invalidation')
                pinned.remove(e.token)
            else:
                require(e.token not in pinned and e.token in erased, 'release before unpin/erase')
                del held[e.token]
                erased.remove(e.token)
        elif e.kind == 'publish':
            require(pending is None, 'incomplete preceding invalidation')
            if e.frames:
                require(held.get(e.token) == (e.pid, e.frames) and e.token in pinned, 'published owner/pin')
            else:
                require(e.token == 0, 'empty publication token')
            pending = (e.pid, tuple(range(max(len(visible), len(e.frames)))))
            visible = e.frames
            selected = e.pid
        elif e.kind == 'invalidate':
            require(pending == (e.pid, e.frames) and e.token == 0, 'invalidation interval')
            cached = visible
            pending = None
        else:
            require(False, 'unknown event')
    require(pending is None, 'uncompleted publication')
    require(visible == cached == after.frames(), 'selected translation state')
    require(selected == after.active or (after.active is None and not after.processes), 'selected process')
    require(held == {e.token: (e.pid, e.frames) for e in after.extents}, 'retained owners')
    require(pinned == set(held) and next_token == after.next_token, 'final pins/history')
    require(before.geometry == after.geometry, 'geometry changed')
    return len(events)
