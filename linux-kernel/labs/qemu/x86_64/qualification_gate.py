#!/usr/bin/env python3
"""Retain lab artifacts on repository disk, independently of identity gates."""
import argparse
import hashlib
import json
from pathlib import Path
import runpy
import subprocess
import sys
import tempfile
import traceback
import uuid

LAB=Path(__file__).resolve().parent
REPOSITORY=LAB.parents[3]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',required=True,type=Path)
    parser.add_argument('--gate',required=True)
    parser.add_argument('arguments',nargs=argparse.REMAINDER)
    args=parser.parse_args()
    root=args.output.resolve()
    root.mkdir(parents=True)
    script=LAB/args.gate
    assert script.parent==LAB and script.suffix=='.py' and script.is_file()
    # Both paths are authorized writable locations. The alias keeps debugger
    # Unix socket names short; all large files remain on repository disk.
    alias=Path('/tmp')/('tq-'+uuid.uuid4().hex[:12])
    alias.symlink_to(root,target_is_directory=True)
    scratch=alias/'scratch'
    scratch.mkdir()
    record=dict(schema='topal.retained-lab-command/1',complete=False,passed=False,
        head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=REPOSITORY,text=True).strip(),
        tree=subprocess.check_output(['git','rev-parse','HEAD^{tree}'],cwd=REPOSITORY,text=True).strip(),
        gate=str(script.relative_to(REPOSITORY)),gate_sha256=digest(script),
        wrapper_sha256=digest(Path(__file__)),arguments=args.arguments,
        short_alias=str(alias),disk_output=str(root),temporary_directories=[])
    path=root/'retention.json'

    def save():
        path.write_text(json.dumps(record,indent=2)+'\n')

    original=tempfile.TemporaryDirectory

    class RetainedTemporaryDirectory(original):
        def __init__(self,*positional,**keywords):
            positional=list(positional)
            if len(positional)>2:
                positional[2]=str(scratch)
                keywords.pop('dir',None)
            else:
                keywords['dir']=str(scratch)
            super().__init__(*positional,**keywords)
            self._finalizer.detach()
            record['temporary_directories'].append(dict(alias=self.name,
                disk_path=str(Path(self.name).resolve())))
            save()

        def cleanup(self):
            # The containing qualification owns retirement of these artifacts.
            self._finalizer.detach()

    save()
    original_arguments=sys.argv
    gate_arguments=args.arguments[1:] if args.arguments[:1]==['--'] else args.arguments
    sys.argv=[str(script),*gate_arguments]
    tempfile.TemporaryDirectory=RetainedTemporaryDirectory
    status=0
    try:
        runpy.run_path(str(script),run_name='__main__')
    except SystemExit as error:
        status=error.code if isinstance(error.code,int) else int(error.code is not None)
        if error.code is not None and not isinstance(error.code,int):
            print(error.code,file=sys.stderr)
    except BaseException:
        status=1
        traceback.print_exc()
    finally:
        tempfile.TemporaryDirectory=original
        sys.argv=original_arguments
        record.update(complete=True,passed=status==0,exit_code=status)
        save()
    return status


if __name__=='__main__':
    raise SystemExit(main())
