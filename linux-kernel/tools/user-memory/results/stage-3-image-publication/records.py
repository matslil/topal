"""Resolve selected evidence deltas and immutable inherited packed inputs."""
import gzip
import importlib.util
import json
import os
from pathlib import Path
import hashlib

def digest(raw):return hashlib.sha256(raw).hexdigest()
def read(path):return json.loads(Path(path).read_bytes())
def unpack(root,row):
    path=(root/row['snapshot']).resolve()
    assert path.is_relative_to(root.parents[4])
    packed=path.read_bytes()
    assert digest(packed)==row['snapshot_sha256']
    raw=gzip.decompress(packed)
    assert len(raw)==row['bytes'] and digest(raw)==row['sha256']
    return raw

def inputs(root):
    root=Path(root).resolve()
    meta=read(root/'inputs.json')
    def base(key):
        row=meta[key]
        path=(root/row['path']).resolve()
        assert path.is_relative_to(root.parents[4])
        raw=path.read_bytes()
        assert len(raw)==row['bytes'] and digest(raw)==row['sha256']
        return path
    base_inputs=base('base_inputs')
    reader=base('base_reader')
    manifest=base('base_manifest')
    for name,row in read(manifest)['files'].items():
        raw=(manifest.parent/name).read_bytes()
        assert len(raw)==row['bytes'] and digest(raw)==row['sha256']
    spec=importlib.util.spec_from_file_location('prior_publication_records',reader)
    module=importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    sources,artifacts=module.inputs(base_inputs.parent)
    def rebase(rows):
        return {name:dict(row,snapshot=os.path.relpath((base_inputs.parent/row['snapshot']).resolve(),root))
            for name,row in rows.items()}
    sources,artifacts=rebase(sources),rebase(artifacts)
    sources.update(meta['sources'])
    artifacts.update(meta['artifacts'])
    return sources,artifacts
