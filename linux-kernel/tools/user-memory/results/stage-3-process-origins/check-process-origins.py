"""Validate source-map instrumentation with the existing bounded post-checks."""
from pathlib import Path
source=Path('linux-kernel/.cache/x86_64/check-startup-source.py').read_text()
source=source.replace('tk283-startup-post-','tk283-process-origins-post-')
source=source.replace("'src/topal-compiler/src/systems_artifact_static_image.rs',",
    "'src/topal-compiler/src/systems_frame_root_tests.rs',\n"
    "     'src/topal-compiler/src/systems_provider_emit_translation.rs',\n"
    "     'src/topal-compiler/src/systems_provider_emit_publication.rs',\n"
    "     'src/topal-compiler/src/systems_user_extent_inventory.rs',\n"
    "     'src/topal-compiler/src/systems_x86_function.rs',")
exec(compile(source,__file__,'exec'))
