"""Record the selected source-built exec-vector native test under its cgroup."""
from pathlib import Path

source=Path('linux-kernel/.cache/tk283-source-retry.py').read_text()
source=source.replace('tk283-source-run-', 'tk283-source-exec-run-')
source=source.replace('owned_source_fixture_checks_native_user_buffer_boundaries',
                      'owned_source_fixture_checks_exec_vector_boundaries')
exec(compile(source, __file__, 'exec'))
