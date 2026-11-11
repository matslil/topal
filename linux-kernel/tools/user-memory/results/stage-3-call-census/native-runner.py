"""Record bounded source-issued descriptor-readiness qualification."""
from pathlib import Path
source=Path('linux-kernel/.cache/tk283-source-retry.py').read_text()
source=source.replace('tk283-source-run-','tk283-source-poll-run-')
source=source.replace('owned_source_fixture_checks_native_user_buffer_boundaries',
                      'owned_source_fixture_checks_poll_transfer_boundaries')
exec(compile(source,__file__,'exec'))
