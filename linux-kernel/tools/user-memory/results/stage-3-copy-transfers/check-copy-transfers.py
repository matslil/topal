"""Run the established bounded compiler/workspace checks plus REP controls."""
from pathlib import Path
source=Path('linux-kernel/.cache/x86_64/check-fixed-transfers.py').read_text()
source=source.replace("source.replace('tk283-startup-post-', 'tk283-fixed-transfers-post-')",
    "source.replace('tk283-startup-post-', 'tk283-copy-transfers-post-')")
exec(compile(source,__file__,'exec'))
