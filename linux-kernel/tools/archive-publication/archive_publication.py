#!/usr/bin/env python3
"""Keep pinned external binary dependencies outside Git without dropping proof."""
import argparse
import ast
import base64
import gzip
import hashlib
import json
import os
from pathlib import Path

ROOT=Path(__file__).resolve().parents[3]
CACHE=ROOT/'linux-kernel/.cache/qualification-external'
SCHEMA='topal.qualification-external-payload-projection/1'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def encode(data):
    return base64.b64encode(gzip.compress(data,mtime=0)).decode('ascii')


def parts(data,payload):
    result=[]
    offset=0
    while True:
        at=data.find(payload,offset)
        if at<0:
            if offset<len(data): result.append(dict(literal_gzip_base64=encode(data[offset:])))
            break
        if at>offset: result.append(dict(literal_gzip_base64=encode(data[offset:at])))
        result.append(dict(external_payload=True))
        offset=at+len(payload)
    assert any(part.get('external_payload') for part in result)
    return result


def reconstruct(recipe,payload):
    output=[]
    for part in recipe:
        assert set(part) in ({'external_payload'},{'literal_gzip_base64'})
        if 'external_payload' in part:
            assert part['external_payload'] is True
            output.append(payload)
        else:
            output.append(gzip.decompress(base64.b64decode(part['literal_gzip_base64'],validate=True)))
    return b''.join(output)


def write_json(path,value):
    path.write_text(json.dumps(value,indent=2)+'\n')


def record_path(archive):
    paths=[archive/name for name in ('qualification.json','archive.json') if (archive/name).is_file()]
    assert len(paths)==1,('ambiguous archive record',archive)
    return paths[0]


def index_field(record):
    fields=[name for name in ('archive_index_sha256','index_sha256') if name in record]
    assert len(fields)==1
    return fields[0]


def publish(archive,payload,cache,expected_payload_sha256):
    archive=archive.resolve()
    cache=cache.resolve()
    assert payload and digest(payload)==expected_payload_sha256,'wrong pinned external payload'
    assert not (archive/'publication-projection.json').exists(),'archive already projected'
    index_path=archive/'files.json'
    original_index=index_path.read_bytes()
    index=json.loads(original_index)
    record_file=record_path(archive)
    original_record=record_file.read_bytes()
    record=json.loads(original_record)
    field=index_field(record)
    assert record[field]==digest(original_index)
    namespace_digest=None
    if 'namespace_record_sha256' in record:
        namespace=archive.parent/'namespace-consumer'
        assert digest((namespace/'raw-qualification.json').read_bytes())==record['namespace_record_sha256']
        namespace_digest=digest((namespace/'qualification.json').read_bytes())
    objects={}
    entries_by_digest={}
    for name,entry in index['files'].items():
        key=entry['sha256']
        assert len(key)==64 and all(char in '0123456789abcdef' for char in key)
        assert entry['blob']=='blobs/'+key+'.gz','only fresh raw archives may be projected'
        entries_by_digest.setdefault(key,[]).append(entry)
        if key in objects: continue
        path=archive/entry['blob']
        data=gzip.decompress(path.read_bytes())
        assert len(data)==entry['bytes'] and digest(data)==key,name
        objects[key]=dict(path=path,bytes=len(data))
        if payload in data:
            recipe=parts(data,payload)
            assert reconstruct(recipe,payload)==data
            objects[key]['parts']=recipe
    for key,entries in entries_by_digest.items():
        assert all(entry['bytes']==objects[key]['bytes'] for entry in entries)
    projection=dict(schema=SCHEMA,policy='TK-BINARY-001',
        external_payload_sha256=expected_payload_sha256,external_payload_bytes=len(payload),
        raw_index_sha256=digest(original_index),raw_record_sha256=digest(original_record),
        raw_record_filename='raw-'+record_file.name,
        publication_tool_sha256=digest(Path(__file__).read_bytes()),
        logical_files=len(index['files']),logical_blobs=len(objects),external_blobs={})
    cache.mkdir(parents=True,exist_ok=True)
    for key,obj in objects.items():
        if 'parts' not in obj: continue
        destination=cache/(key+'.gz')
        if destination.exists():
            assert digest(gzip.decompress(destination.read_bytes()))==key
        projection['external_blobs'][key]=dict(bytes=obj['bytes'],parts=obj['parts'],
            blob=os.path.relpath(destination,archive))
        for entry in entries_by_digest[key]:
            entry['blob']=os.path.relpath(destination,archive)
            entry['external_payload_sha256']=expected_payload_sha256
    # The original producer records and index remain byte-exact audit inputs.
    (archive/'raw-files.json').write_bytes(original_index)
    (archive/projection['raw_record_filename']).write_bytes(original_record)
    write_json(archive/'publication-projection.json',projection)
    for key,obj in objects.items():
        if 'parts' in obj:
            obj['path'].replace(cache/(key+'.gz'))
    write_json(index_path,index)
    record[field]=digest(index_path.read_bytes())
    record['publication_projection_sha256']=digest((archive/'publication-projection.json').read_bytes())
    record['publication_raw_record_sha256']=digest(original_record)
    record['publication_external_blobs']=len(projection['external_blobs'])
    record['publication_repository_blobs']=len(objects)-len(projection['external_blobs'])
    # A full-qualification record points to the published namespace view while
    # its preserved original still binds the original producer's namespace record.
    if namespace_digest is not None:
        record['namespace_record_sha256']=namespace_digest
    write_json(record_file,record)
    return projection


def hydrate(archive,payload,cache):
    archive=archive.resolve()
    cache=cache.resolve()
    projection_path=archive/'publication-projection.json'
    projection_bytes=projection_path.read_bytes()
    projection=json.loads(projection_bytes)
    assert projection['schema']==SCHEMA
    assert digest(payload)==projection['external_payload_sha256']
    assert len(payload)==projection['external_payload_bytes']
    assert digest((archive/'raw-files.json').read_bytes())==projection['raw_index_sha256']
    assert digest((archive/projection['raw_record_filename']).read_bytes())==projection['raw_record_sha256']
    record=json.loads(record_path(archive).read_text())
    assert record['publication_projection_sha256']==digest(projection_bytes)
    index_bytes=(archive/'files.json').read_bytes()
    index=json.loads(index_bytes)
    assert record[index_field(record)]==digest(index_bytes)
    expected={entry['sha256'] for entry in index['files'].values() if 'external_payload_sha256' in entry}
    assert expected==set(projection['external_blobs'])
    cache.mkdir(parents=True,exist_ok=True)
    for key,entry in projection['external_blobs'].items():
        data=reconstruct(entry['parts'],payload)
        assert digest(data)==key and len(data)==entry['bytes']
        destination=cache/(key+'.gz')
        assert (archive/entry['blob']).resolve()==destination
        for indexed in index['files'].values():
            if indexed['sha256']==key:
                assert indexed['bytes']==entry['bytes'] and indexed['blob']==entry['blob']
                assert indexed['external_payload_sha256']==projection['external_payload_sha256']
        if destination.exists():
            assert gzip.decompress(destination.read_bytes())==data
        else:
            destination.write_bytes(gzip.compress(data,mtime=0))
    return projection


def pinned_payload_sha256():
    tree=ast.parse((ROOT/'linux-kernel/labs/qemu/x86_64/topal_gate.py').read_text())
    values=[ast.literal_eval(node.value) for node in tree.body if isinstance(node,ast.Assign)
        and any(isinstance(target,ast.Name) and target.id=='PINNED_STATIC_BUSYBOX_SHA256'
            for target in node.targets)]
    assert len(values)==1 and isinstance(values[0],str)
    return values[0]


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('operation',choices=('publish','hydrate'))
    parser.add_argument('--archive',required=True,action='append',type=Path)
    parser.add_argument('--payload',required=True,type=Path)
    args=parser.parse_args()
    payload=args.payload.read_bytes()
    for archive in args.archive:
        if args.operation=='publish':
            result=publish(archive,payload,CACHE,pinned_payload_sha256())
        else:
            result=hydrate(archive,payload,CACHE)
        print(args.operation.upper()+'_PASS '+str(archive)+' external_blobs='+str(len(result['external_blobs'])),flush=True)
