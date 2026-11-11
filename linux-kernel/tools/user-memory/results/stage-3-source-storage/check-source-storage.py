"""Record bounded source-storage post-checks."""
from pathlib import Path
source=Path('linux-kernel/.cache/x86_64/check-startup-source.py').read_text()
source=source.replace('tk283-startup-post-','tk283-storage-post-')
source=source.replace("'src/topal-compiler/src/systems_artifact_static_image.rs',",
                      "'src/topal-compiler/src/systems_frame_root_tests.rs',")
exec(compile(source,__file__,'exec'))
