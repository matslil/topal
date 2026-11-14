"""Record ordinary process-owned file/directory buffer qualification."""
from pathlib import Path
source=Path('linux-kernel/.cache/tk283-source-retry.py').read_text()
source=source.replace('tk283-source-run-','tk283-source-storage-run-')
source=source.replace('owned_source_fixture_checks_native_user_buffer_boundaries',
                      'owned_source_fixture_checks_storage_transfer_boundaries')
exec(compile(source,__file__,'exec'))
