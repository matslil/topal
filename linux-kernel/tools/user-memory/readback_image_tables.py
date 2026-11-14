"""Replay all owned inactive table construction bytes and lifetime snapshots."""
import json
from pathlib import Path
import sys
from readback_image_backing import inspect

if __name__ == '__main__':
    build = Path(sys.argv[1]).resolve()
    assert build.is_relative_to(Path(__file__).resolve().parents[2])
    result = inspect(build,tables=True)
    (build/'image-tables-readback.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result))
