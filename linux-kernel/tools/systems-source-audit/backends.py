"""Compare TK277 extraction with its pinned qualified predecessor; run at repository root."""

from pathlib import Path
from collections import Counter
import hashlib, re, subprocess, tempfile
from rust_items import items, body, masked_rust

if not __debug__:
    raise RuntimeError("Run this qualification without Python optimization")


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
    for owner, namespace in [("src/topal-compiler/src/systems_artifact.rs","compiler::systems_artifact"),("src/topal-compiler/src/systems_provider.rs","compiler::systems_provider"),("src/topal-compiler/src/systems_provider_object.rs","compiler::systems_provider_object")]:
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

expected_new = {
    'compiler::systems_provider_object::emit_static_process_'+part
    for part in ['start','read_exec','files','output','completion']
} | {
    'compiler::systems_provider_object::tests::audit_provider_'+part
    for part in ['devices','bootstrap','entropy_authority','process_entry',
                 'process_syscalls','image_mapping','user_execution','user_completion']
}
assert added == expected_new, added ^ expected_new
assert not removed, removed
assert changed == {
    'compiler::systems_provider_object::static_image_process_enter',
    'compiler::systems_provider_object::tests::emits_only_the_sealed_provider_instruction_sequences',
}, changed
assert old_declarations == new_declarations
print('PASS',len(old)-len(changed),'unchanged backend function token/literal fingerprints;',
      sum(old_declarations.values()),'unchanged declarations/imports/macros/type paths and module gates')

def normalize_body(value):
    # Equal formatting context preserves literal contents and normalizes only
    # Rust formatting, including optional trailing commas.
    with tempfile.TemporaryDirectory(prefix='topal-tape-audit-',dir='/tmp') as directory:
        path=Path(directory)/'tape.rs'
        path.write_text('fn audit() {\n'+value+'\n}')
        subprocess.run(['rustfmt','--edition','2024',str(path)],check=True)
        return canonical(body(path.read_text())[1])

key='compiler::systems_provider_object::static_image_process_enter'
old_body=body(old[key])[1]
constant_end=old_body.index('    let mut code = X86FunctionEncoder::default();')
old_constants=old_body[:constant_end]
init_end=old_body.index('\n',constant_end)+1
tape=old_body[init_end:old_body.rindex('    code.finish_linkable()')]
new_tape=''
for part in ['start','read_exec','files','output','completion']:
    value=body(new['compiler::systems_provider_object::emit_static_process_'+part])[1]
    for match in list(re.finditer(r'^    const (?:OBJECT|TAG|VALUE|TERMINATOR|CONTEXT_QWORDS):[^\n]+\n',value,re.M)):
        assert match[0] in old_constants,match[0]
    value=re.sub(r'^    const (?:OBJECT|TAG|VALUE|TERMINATOR|CONTEXT_QWORDS):[^\n]+\n','',value,flags=re.M)
    assert value.rstrip().endswith('Ok(())')
    new_tape+=value[:value.rindex('    Ok(())')]
assert normalize_body(tape.replace('&mut code','code')) == normalize_body(new_tape)
print('PASS exact ordered process instruction/control/label/relocation emission tape and constant values')

expected_caller = '''
    let mut code = X86FunctionEncoder::default();
    emit_static_process_start(&mut code, source_bound, file_open_plan,
        source_numeric, source_image, source_dispatch, dispatch_policy,
        source_identity, retained_identity)?;
    emit_static_process_read_exec(&mut code, source_image)?;
    emit_static_process_files(&mut code)?;
    emit_static_process_output(&mut code)?;
    emit_static_process_completion(&mut code, source_bound, file_open_plan,
        source_numeric, source_image, source_dispatch, source_identity,
        retained_identity, structured_identity, source_position)?;
    code.finish_linkable()
'''
assert normalize_body(body(new[key])[1]) == normalize_body(expected_caller)
print('PASS original encoder, exact phase order and unchanged argument forwarding')

key='compiler::systems_provider_object::tests::emits_only_the_sealed_provider_instruction_sequences'
old_body=body(old[key])[1]
# The wrapper retains its original generated object and ELF reader. Helpers
# carry existing shared observations by return value/reference only.
start=old_body.index('        for (name, expected) in [')
old_tape=old_body[start:]
new_tape=''
for part,result in [('devices',None),('bootstrap','data'),('entropy_authority',None),
                    ('process_entry','(process_enter, process_restore, process_fixture_image)'),
                    ('process_syscalls',None),('image_mapping',None),
                    ('user_execution','entry_payload'),('user_completion',None)]:
    value=body(new['compiler::systems_provider_object::tests::audit_provider_'+part])[1]
    if result:
        end=value.rfind(result)
        assert not value[end+len(result):].strip()
        value=value[:end]
    new_tape+=value
assert normalize_body(old_tape)==normalize_body(new_tape)
print('PASS every ordered instruction assertion and diagnostic literal retained in eight audit helpers')
