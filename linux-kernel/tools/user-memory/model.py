"""Independent owned-extent oracle for TOPAL-SYSTEMS-USER-EXTENT-001.

These immutable observations are model inputs, never kernel capabilities or a
production authority-construction API. No candidate buffer is dereferenced.
The first qualified target subset has LA48, four levels and 4-KiB leaves.
"""
from dataclasses import dataclass
from enum import Enum
from types import MappingProxyType
from typing import Mapping

PAGE = 4096
WORD_MAX = (1 << 64) - 1
USER_END = 1 << 47
PHYSICAL_END = 1 << 30
ADDRESS_BITS = ((1 << 52) - 1) & -PAGE
NX = 1 << 63
# Conservative target subset: P/RW/US and hardware A/D; other formats need
# separate qualification. NX affects instruction fetch, not this data extent.
ENTRY_BITS = ADDRESS_BITS | NX | 0x67


class Status(Enum):
    EMPTY = 'empty'
    ADMITTED = 'admitted'
    RANGE = 'invalid-range'
    ABSENT = 'absent'
    PERMISSION = 'permission'
    UNSUPPORTED = 'unsupported-format-or-mode'


class InvariantViolation(RuntimeError):
    """A trusted owner/context invariant failed; not a normal user-pointer miss."""


@dataclass(frozen=True)
class Verdict:
    status: Status
    checked_pages: int = 0


@dataclass(frozen=True)
class Table:
    level: int
    words: tuple[int, ...]

    def __post_init__(self):
        if len(self.words) != 512 or type(self.words) is not tuple:
            raise ValueError('a table snapshot has exactly 512 immutable words')
        if any(type(word) is not int or not 0 <= word <= WORD_MAX for word in self.words):
            raise ValueError('complete unsigned table words required')


@dataclass(frozen=True)
class Backing:
    physical_page: int
    writable: bool = True
    live: bool = True
    pins: int = 1


@dataclass(frozen=True)
class Observation:
    root: int
    current_root: int
    entry: int
    current_entry: int
    tables: Mapping[int, Table]
    backing: Mapping[int, Backing]
    pinned_tables: frozenset[int]
    privileged: bool = True
    critical: bool = True
    stable: bool = True
    mode: str = 'la48-small'

    def __post_init__(self):
        # Snapshot caller containers as well as table bytes. Descriptive input
        # immutability is not evidence of actual hardware mapping stability.
        object.__setattr__(self, 'tables', MappingProxyType(dict(self.tables)))
        object.__setattr__(self, 'backing', MappingProxyType(dict(self.backing)))
        object.__setattr__(self, 'pinned_tables', frozenset(self.pinned_tables))


def admit(observed: Observation, start: int, length: int, *, write=False,
          reads: list[tuple[int, int]] | None = None) -> Verdict:
    """Check the entire range; trace table reads separately from the verdict.

    Ordinary range/permission failures return a verdict. Invalid trusted
    ownership raises InvariantViolation; callers cannot convert it to success.
    """
    if (type(start) is not int or type(length) is not int
            or not 0 <= start <= WORD_MAX or not 0 <= length <= WORD_MAX
            or start + length > WORD_MAX):
        return Verdict(Status.RANGE)
    if length == 0:
        return Verdict(Status.EMPTY)
    end = start + length
    if start >= USER_END or end > USER_END:
        return Verdict(Status.RANGE)
    if (type(observed.root) is not int or type(observed.current_root) is not int
            or type(observed.entry) is not int or type(observed.current_entry) is not int
            or not 0 <= observed.root <= WORD_MAX or not 0 <= observed.current_root <= WORD_MAX
            or not 0 < observed.entry <= WORD_MAX or not 0 < observed.current_entry <= WORD_MAX
            or observed.root != observed.current_root
            or observed.entry != observed.current_entry or not observed.privileged
            or not observed.critical or not observed.stable):
        raise InvariantViolation('exact live critical entry/address space required')
    if observed.mode != 'la48-small':
        return Verdict(Status.UNSUPPORTED)
    count = 0
    for page in range(start & -PAGE, end, PAGE):
        table_page = observed.root
        for level, shift in [(4, 39), (3, 30), (2, 21), (1, 12)]:
            # Check the whole page before any word fetch. An in-window numeric
            # pointer alone cannot establish table ownership or its level role.
            if (table_page % PAGE or not 0 <= table_page <= PHYSICAL_END - PAGE
                    or table_page not in observed.pinned_tables):
                raise InvariantViolation('unowned or inaccessible table')
            table = observed.tables.get(table_page)
            if table is None or table.level != level:
                raise InvariantViolation('wrong or missing table owner role')
            index = (page >> shift) % 512
            if reads is not None:
                reads.append((table_page, index))
            word = table.words[index]
            if not word & 1:
                return Verdict(Status.ABSENT, count)
            if word & ~ENTRY_BITS:
                return Verdict(Status.UNSUPPORTED, count)
            required = 7 if write else 5
            if word & required != required:
                return Verdict(Status.PERMISSION, count)
            physical = word & ADDRESS_BITS
            if level != 1:
                table_page = physical
                continue
            owner = observed.backing.get(page)
            if (owner is None or not owner.live or owner.pins <= 0
                    or owner.physical_page != physical
                    or physical % PAGE or not 0 <= physical <= PHYSICAL_END - PAGE
                    or physical in observed.pinned_tables or physical in observed.tables):
                raise InvariantViolation('leaf lacks distinct live pinned user backing')
            if write and not owner.writable:
                return Verdict(Status.PERMISSION, count)
        count += 1
    return Verdict(Status.ADMITTED, count)
