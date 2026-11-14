"""Portable image-role/subdivision model, not a native mapper or authority.

TOPAL-SYSTEMS-KERNEL-IMAGE-PLAN-001 / OWNERS-001. Numeric descriptions do not
create image/root/table owners, a publication witness or a protected context.
"""
from dataclasses import dataclass


def require(ok, message):
    if not ok:
        raise ValueError(message)


def integer(value):
    return type(value) is int


@dataclass(frozen=True)
class Rights:
    read: bool
    write: bool
    execute: bool
    user: bool = False

    def __post_init__(self):
        require(all(type(value) is bool for value in self.bits()), 'rights must be Boolean')

    def bits(self):
        return self.read, self.write, self.execute, self.user

    def contained_in(self, other):
        return all(not actual or allowed for actual, allowed in zip(self.bits(), other.bits()))


CODE = Rights(True, False, True)
IMMUTABLE = Rights(True, False, False)
MUTABLE = Rights(True, True, False)
ROLES = dict(code=CODE, immutable=IMMUTABLE, mutable=MUTABLE, relro=IMMUTABLE)


@dataclass(frozen=True)
class Geometry:
    minimum: int
    large: int
    limit: int

    def __post_init__(self):
        require(all(integer(value) and value > 0 for value in [self.minimum, self.large, self.limit]), 'positive geometry')
        require(self.minimum & (self.minimum - 1) == 0 and self.large & (self.large - 1) == 0,
                'power-of-two geometry')
        require(self.large >= self.minimum and self.large % self.minimum == 0, 'compatible geometry')


@dataclass(frozen=True)
class Region:
    start: int
    end: int
    role: str
    finalized: bool = False


@dataclass(frozen=True)
class Part:
    start: int
    end: int
    rights: Rights


@dataclass(frozen=True)
class Plan:
    geometry: Geometry
    parts: tuple
    split_blocks: tuple


def partition(regions, geometry):
    """Round physical role extents and find necessarily subdivided large blocks.

    Only image extents are described. Holes mean preserve/inspect separately,
    not an implicit mapping or new rights. Split counts are NOT total backing.
    Interior uniform blocks are symbolic, so huge extents are not enumerated.
    """
    require(isinstance(geometry, Geometry), 'qualified geometry description required')
    rounded = []
    for region in regions:
        require(isinstance(region, Region) and type(region.role) is str and region.role in ROLES, 'known image role')
        require(integer(region.start) and integer(region.end)
                and 0 <= region.start < region.end <= geometry.limit, 'bounded nonempty region')
        require(type(region.finalized) is bool, 'finalization description must be Boolean')
        require(region.role != 'relro' or region.finalized, 'relocation initialization incomplete')
        start = region.start // geometry.minimum * geometry.minimum
        end = (region.end + geometry.minimum - 1) // geometry.minimum * geometry.minimum
        require(end <= geometry.limit, 'page rounding exceeds address limit')
        rounded.append(Part(start, end, ROLES[region.role]))
    require(rounded, 'image plan must not be empty')
    edges = sorted({edge for part in rounded for edge in [part.start, part.end]})
    parts = []
    for start, end in zip(edges, edges[1:]):
        active = {part.rights for part in rounded if part.start < end and start < part.end}
        require(len(active) <= 1, 'incompatible roles share a minimum granule')
        if not active:
            continue
        rights = next(iter(active))
        if parts and parts[-1].end == start and parts[-1].rights == rights:
            parts[-1] = Part(parts[-1].start, end, rights)
        else:
            parts.append(Part(start, end, rights))
    candidates = {edge // geometry.large * geometry.large for part in parts
                  for edge in [part.start, part.end - 1]}
    splits = []
    for base in sorted(candidates):
        end = base + geometry.large
        covering = [part for part in parts if part.start < end and base < part.end]
        uniform = any(part.start <= base and end <= part.end for part in covering)
        if not uniform:
            splits.append(base)
    return Plan(geometry, tuple(parts), tuple(splits))


@dataclass(frozen=True)
class View:
    root: str
    virtual: int
    physical: int
    size: int
    rights: Rights
    primary: bool
    memory_kind: str = 'normal'


def check_views(plan, roots, views):
    """Check supplied primary coverage and alias compatibility, not live owners.

    The caller must supply the actual complete root/view set in native work.
    This model cannot discover omitted roots, aliases, CPUs, DMA or stale TLBs.
    It checks physical image coverage; native entry/virtual-placement binding
    remains a separate obligation of the qualified image plan.
    """
    require(roots and all(type(root) is str and root for root in roots)
            and len(set(roots)) == len(roots), 'distinct declared roots')
    require(isinstance(plan, Plan), 'image plan required')
    geometry = plan.geometry
    require(isinstance(geometry, Geometry) and plan.parts, 'nonempty typed plan')
    previous = 0
    for part in plan.parts:
        require(isinstance(part, Part) and integer(part.start) and integer(part.end)
                and previous <= part.start < part.end <= geometry.limit
                and part.start % geometry.minimum == part.end % geometry.minimum == 0
                and part.rights in [CODE, IMMUTABLE, MUTABLE], 'valid ordered role partition')
        previous = part.end
    checked = []
    for view in views:
        require(isinstance(view, View) and view.root in roots, 'unknown root')
        require(isinstance(view.rights, Rights) and type(view.primary) is bool, 'typed view')
        require(all(integer(value) for value in [view.virtual, view.physical, view.size])
                and view.size > 0 and view.virtual >= 0 and view.physical >= 0
                and max(view.virtual, view.physical) + view.size <= geometry.limit, 'bounded view')
        require(view.virtual % geometry.minimum == view.physical % geometry.minimum == view.size % geometry.minimum == 0,
                'view must describe complete mapping granules')
        for other in checked:
            if view.root == other.root and view.virtual < other.virtual + other.size and other.virtual < view.virtual + view.size:
                require(view.physical - view.virtual == other.physical - other.virtual
                        and view.rights == other.rights and view.memory_kind == other.memory_kind,
                        'conflicting virtual mappings in one root')
        for part in plan.parts:
            if view.physical < part.end and part.start < view.physical + view.size:
                require(view.memory_kind == 'normal' and view.rights.contained_in(part.rights), 'incompatible image alias')
                if view.primary:
                    require(view.rights == part.rights, 'primary view lacks required rights')
        checked.append(view)
    for root in roots:
        for part in plan.parts:
            spans = sorted((max(part.start, view.physical), min(part.end, view.physical + view.size))
                           for view in checked if view.root == root and view.primary
                           and view.physical < part.end and part.start < view.physical + view.size)
            cursor = part.start
            for start, end in spans:
                require(start <= cursor, 'primary image coverage has a hole')
                cursor = max(cursor, end)
            require(cursor == part.end, 'primary image coverage incomplete')
