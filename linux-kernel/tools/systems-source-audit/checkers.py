"""Compare TK277 extraction with its pinned qualified predecessor; run at repository root."""

from pathlib import Path
from collections import Counter
import hashlib, re, subprocess, tempfile
from rust_items import items, body, masked_rust

BASE = '73e17ab5a50d34561b71db6fb835ed0aae31217f'
def canonical(text):
    literals = []
    value = masked_rust(text, literals)
    for start, end, literal in reversed(literals):
        value = value[:start] + 'LITERAL_'+hashlib.sha256(literal.encode()).hexdigest() + value[end:]
    return re.sub(r'\s+', '', value)

def inventory(baseline):
    functions, declarations = {}, Counter()
    def load(path):
        if not baseline: return path.read_text()
        return subprocess.run(['rtk','proxy','git','show',BASE+':'+str(path)],
                              check=True,capture_output=True,text=True).stdout
    def walk(path, namespace, contents=None):
        for item in items(load(path) if contents is None else contents):
            mask = masked_rust(item)
            if mask.lstrip().startswith('include!'):
                name = re.search(r'include!\(\s*"([^"]+)"\s*\)', item)[1]
                walk(path.parent/name, namespace)
                continue
            opening = mask.find('{')
            header = mask if opening < 0 else mask[:opening]
            function = re.search(r'\bfn\s+([A-Za-z_][A-Za-z_0-9]*)', header)
            if function:
                key = namespace+'::'+function[1]
                assert key not in functions, key
                functions[key] = item
                continue
            module = re.search(r'\bmod\s+([A-Za-z_][A-Za-z_0-9]*)\s*$', header)
            if module and opening >= 0:
                head, inside, tail = body(item)
                declarations[namespace+'::MODULE::'+canonical(head)] += 1
                walk(path, namespace+'::'+module[1], inside)
                continue
            implementation = re.search(r'\bimpl\s+.+', header, re.S)
            if implementation and opening >= 0:
                head, inside, tail = body(item)
                # Reopening one inherent impl preserves its associated items.
                scope = canonical(implementation[0])
                walk(path, namespace+'::'+scope, inside)
                continue
            declarations[namespace+'::'+canonical(item)] += 1
    for owner, namespace in [('src/topal-language/src/systems.rs','language::systems'),
                              ('src/topal-semantics/src/systems.rs','semantics::systems')]:
        walk(Path(owner), namespace)
    return functions, declarations

old, old_declarations = inventory(True)
new, new_declarations = inventory(False)
added = new.keys()-old.keys()
removed = old.keys()-new.keys()
changed = {name for name in old.keys() & new.keys() if old[name] != new[name]}
candidate = sorted(name for name in changed if canonical(old[name]) != canonical(new[name]))
with tempfile.TemporaryDirectory(prefix='topal-item-audit-', dir='/tmp') as directory:
    path = Path(directory)/'audit.rs'
    entries = [(name, source) for name in candidate for source in [old[name], new[name]]]
    path.write_text('\n'.join('impl Audit%06d {\n%s\n}' % (index, value)
                              for index, (_, value) in enumerate(entries)))
    subprocess.run(['rustfmt','--edition','2024',str(path)], check=True)
    normalized = []
    for item in items(path.read_text()):
        _, inner, _ = body(item)
        functions = items(inner)
        assert len(functions) == 1
        normalized.append(canonical(functions[0]))
    assert len(normalized) == len(entries)
    changed = {name for index, name in enumerate(candidate)
               if normalized[index*2] != normalized[index*2+1]}
assert added == {'language::systems::analyze_kernel_user_continuation',
                 'language::systems::parse_static_image_marker'}, added
assert not removed, removed
assert changed == {'language::systems::analyze_kernel_context_decision',
                   'language::systems::analyze_static_elf_frame_sequence'}, changed
assert old_declarations == new_declarations, (old_declarations-new_declarations, new_declarations-old_declarations)
print('PASS',len(old)-len(changed),'unchanged function token/literal fingerprints; only two reviewed helper decompositions')
print('PASS',sum(old_declarations.values()),'unchanged declarations/imports/macros/type paths and module gates; no removed functions')
