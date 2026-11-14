"""Read the current evidence delta and its immutable preceding packed inputs."""
import gzip
import hashlib
import json
import os
from pathlib import Path


def digest(raw):return hashlib.sha256(raw).hexdigest()
def read(path):return json.loads(Path(path).read_bytes())


def unpack(root,row):
    path=(root/row['snapshot']).resolve()
    assert path.is_relative_to(root.parents[4])
    packed=path.read_bytes()
    assert digest(packed)==row['snapshot_sha256']
    raw=gzip.decompress(packed)
    assert digest(raw)==row['sha256'] and len(raw)==row['bytes']
    return raw


def inputs(root):
    root=Path(root).resolve()
    meta=read(root/'inputs.json')
    def base(key):
        row=meta[key]
        path=(root/row['path']).resolve()
        assert path.is_relative_to(root.parents[4])
        raw=path.read_bytes()
        assert digest(raw)==row['sha256'] and len(raw)==row['bytes']
        return path,read(path)
    source_path,source=base('base_sources')
    artifact_path,artifacts=base('base_artifacts')
    manifest_path,manifest=base('base_manifest')
    for name,row in manifest['files'].items():
        data=(manifest_path.parent/name).read_bytes()
        assert digest(data)==row['sha256'] and len(data)==row['bytes']
    def rebase(rows,parent):
        return {name:dict(row,snapshot=os.path.relpath((parent/row['snapshot']).resolve(),root))
            for name,row in rows.items()}
    sources=rebase(source['selected_inputs'],source_path.parent)
    sources.update(meta['sources'])
    retained=rebase(artifacts,artifact_path.parent)
    retained.update(meta['artifacts'])
    return sources,retained
