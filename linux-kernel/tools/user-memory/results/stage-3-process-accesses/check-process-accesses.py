"""Record bounded post-checks for selected transfer-guard auditing."""
from pathlib import Path
source = Path('linux-kernel/.cache/x86_64/check-source-serial.py').read_text()
source = source.replace("tk283-serial-post-", "tk283-process-accesses-post-")
base = Path('linux-kernel/.cache/x86_64/check-startup-source.py').read_text()
base = base.replace('tk283-startup-post-', 'tk283-process-accesses-post-')
base = base.replace("'src/topal-compiler/src/systems_artifact_static_image.rs',",
                    "'src/topal-compiler/src/systems_frame_root_tests.rs',")
exec(compile(base, __file__, 'exec'))
