"""Run appropriate bounded post-checks for the scalar admission reader."""
from pathlib import Path
source = Path('linux-kernel/.cache/x86_64/check-startup-source.py').read_text()
source = source.replace('tk283-startup-post-', 'tk283-fixed-transfers-post-')
source = source.replace("'src/topal-compiler/src/systems_artifact_static_image.rs',",
    "'src/topal-compiler/src/systems_frame_root_tests.rs',")
source = source.replace('commands = [',
    "commands = [\n    ['python3','-m','unittest','discover','-s','linux-kernel/tools/user-memory','-v'],")
exec(compile(source, __file__, 'exec'))
