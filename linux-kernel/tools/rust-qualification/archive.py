#!/usr/bin/env python3
"""Retain a completed Rust qualification record and lossless original logs."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import re


def digest(data):
    return hashlib.sha256(data).hexdigest()


def archive(source,output,allow_failed=False):
    record_bytes=(source/'qualification.json').read_bytes()
    record=json.loads(record_bytes)
    assert record['schema']==1 and record['complete']
    assert record['passed'] or allow_failed
    assert [entry['name'] for entry in record['commands']]==['check','test','doc-test']
    logs={}
    for entry in record['commands']:
        assert isinstance(entry['returncode'],int)
        data=(source/entry['log']).read_bytes()
        assert digest(data)==entry['log_sha256']
        logs[entry['name']]=data
    assert record['passed']==all(entry['returncode']==0 for entry in record['commands'])
    pattern=rb'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;'
    counts=[tuple(map(int,match)) for match in re.findall(pattern,logs['test'])]
    assert counts
    if record['passed']:
        assert all(row[1:]==(0,0,0,0) for row in counts)
    tests=sum(row[0] for row in counts)
    assert tests>0
    doc_counts=[tuple(map(int,match)) for match in re.findall(pattern,logs['doc-test'])]
    if record['passed']:
        assert all(row[1:]==(0,0,0,0) for row in doc_counts)
    output.mkdir(parents=True)
    (output/'qualification.json').write_bytes(record_bytes)
    for entry in record['commands']:
        data=logs[entry['name']]
        target=output/(entry['log']+'.gz')
        target.write_bytes(gzip.compress(data,mtime=0))
        assert gzip.decompress(target.read_bytes())==data
    summary=dict(schema='topal.rust-qualification-log-summary/1',head=record['head'],tree=record['tree'],
        passed=record['passed'],passed_tests=tests,failed_tests=sum(row[1] for row in counts),
        ignored_tests=sum(row[2] for row in counts),target_summaries=len(counts),
        doc_tests=sum(row[0] for row in doc_counts),
        qualification_sha256=digest(record_bytes),archiver_sha256=digest(Path(__file__).read_bytes()),
        scope='Rust implementation/all-target/doc qualification only; machine observations remain separate')
    (output/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
    print('RUST_ARCHIVE_SAVED passed='+str(record['passed'])+' tests='+str(tests)+' target_summaries='+str(len(counts)),flush=True)
    return summary


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source',required=True,type=Path)
    parser.add_argument('--output',required=True,type=Path)
    parser.add_argument('--allow-failed',action='store_true')
    args=parser.parse_args()
    archive(args.source.resolve(),args.output.resolve(),args.allow_failed)
