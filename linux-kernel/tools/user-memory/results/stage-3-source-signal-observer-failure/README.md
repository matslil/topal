# Failed signal observer staging assertion

The retained native command exits 101. Its first signal request stops in the
observer before that syscall executes: the observer demands zero for table
bytes `[1408,1792)`. Actual transfer staging occupies `[1408,1664)`; the next
word is the live process umask. The observer incorrectly includes unrelated
owned state in its zero assertion. The correction checks the full 256-byte
staging extent and retains the complete 8192-byte table comparison, including
the umask. No kernel bytes or security requirements are changed.

This failed run remains failed. A later fresh native run must establish the
corrected assertions; this record grants no passing native qualification.
The selected modified-input snapshots, run/resource record, generated observer
and raw logs are retained. This is not an AI safeguard denial, a full producer
census or a production reconstruction. No executable payload is archived.
