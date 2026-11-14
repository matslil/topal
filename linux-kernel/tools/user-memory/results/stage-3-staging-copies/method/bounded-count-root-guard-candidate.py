def bounded_count_initializers(nodes, predecessors, at, roots):
    """Every reaching R15 definition in this selected cwd prelude is 2 or 5."""
    assert at not in roots, ('unbounded cwd count entry',at)
    seen,pending,definitions = set(),list(predecessors[at]),[]
    while pending:
        current = pending.pop()
        if current in seen:
            continue
        assert current in nodes and len(seen) < 64
        seen.add(current)
        raw = bytes.fromhex(nodes[current]['bytes'])
        if raw[:2] == bytes.fromhex('49 bf'):
            value = int.from_bytes(raw[2:],'little')
            assert len(raw) == 10 and value in [2,5]
            definitions.append(dict(offset=current,value=value))
            continue
        assert current not in roots, ('unbounded cwd count entry',current)
        assert ((len(raw) == 6 and raw[0] == 0x0f and 0x80 <= raw[1] <= 0x8f) or
                (len(raw) == 5 and raw[0] == 0xe9) or raw == bytes.fromhex('4c 39 fe') or
                (len(raw) == 7 and raw[:3] == bytes.fromhex('48 81 f8'))), ('unproved cwd count definition',current)
        assert predecessors[current]
        pending.extend(predecessors[current])
    assert {r['value'] for r in definitions} == {2,5}
    return sorted(definitions,key=lambda r:r['offset'])
