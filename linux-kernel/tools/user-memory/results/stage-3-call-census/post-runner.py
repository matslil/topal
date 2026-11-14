"""Record bounded post-checks for the test-only extent-call census."""
from pathlib import Path
source = Path('linux-kernel/.cache/x86_64/check-startup-source.py').read_text()
source = source.replace('tk283-startup-post-', 'tk283-census-post-')
source = source.replace("'src/topal-compiler/src/systems_artifact_static_image.rs',\n     'src/topal-compiler/src/systems_user_source_fixture.rs'", "'src/topal-compiler/src/systems_provider_emit_translation.rs',\n     'src/topal-compiler/src/systems_provider_emit_publication.rs',\n     'src/topal-compiler/src/systems_frame_root_tests.rs',\n     'src/topal-compiler/src/systems_provider_object.rs',\n     'src/topal-compiler/src/systems_user_extent_inventory.rs'")
exec(compile(source, __file__, 'exec'))
