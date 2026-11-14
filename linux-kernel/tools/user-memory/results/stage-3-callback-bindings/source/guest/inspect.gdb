set pagination off
set confirm off
set remotetimeout 10
file "/home/mats/git/topal/linux-kernel/.cache/x86_64/user-extent-source-storage-2282874-1794404608667753907/kernel.elf"
target remote 127.0.0.1:59241
python
token_bytes=336
observation_path='/home/mats/git/topal/linux-kernel/.cache/x86_64/user-extent-source-storage-2282874-1794404608667753907/source-user-extents/observations.json'
component={'base': 5701632, 'command_source_sha256': 'cac5afad81808c0123175a2de8b1e799c70579f9a5ef81445bb3a66433f9e3ef', 'conservative_user_stack_bytes': 1320, 'entry': 5708672, 'exec_source_sha256': None, 'file_source_sha256': '757a45600b38e0746659781ba6e14aa5a90ec9d378cf364fd555299890ef92fa', 'image_sha256': 'd821b3a56eb090602e8e48a9e1b04a127c11625d7ef726381e4aa5d0a4f54199', 'linked_sha256': '8b3170a87c437f162f8aaecd20b06d13dd626c171267bcf3c88090298f44faf6', 'mode': 'storage', 'pipe_source_sha256': None, 'poll_source_sha256': None, 'scope': 'source-built ordinary syscall component in a freshly generated first-party ABI witness; not an arbitrary-program loader or production qualification', 'serial_source_sha256': None, 'shell_source_sha256': '574462c620fb4e2db01bc4140e98377f52d4add8eddc3e4a9fa0428d2a3d8db4', 'signal_source_sha256': None, 'source_sha256': '4350abed34fe56b3961d52ec3d73532a03de56320cf08f84d0cac5688538d70b', 'stack_frame_costs': {'command-cases': 104, 'file-cases': 400, 'shell-cases': 168, 'source-cases': 496, 'startup-cases': 168, 'storage-cases': 168}, 'startup': 5702592, 'startup_call': 5711520, 'startup_source_sha256': '2c5171f997c27d084883f6dce969747c191ab41649864e5c1d73f2367bb574aa', 'storage_source_sha256': '9f1a971d58b7f9b437fd0f159c08fef922241fa1b6007a2e26187b091edd5f3c', 'syscall': 5711472, 'syscall4': 5711488, 'text_bytes': 10080}
immutable_head='7f454c4602010103000000000000000002003e000100000080654a00000000004000000000000000000000000000000000000000400038000b000000000000000100000004000000000000000000000000004000000000000000400000000000e006000000000000e0060000000000000010000000000000010000000500000000100000000000000010400000000000001040000000000085b218000000000085b21800000000000010000000000000010000000400000000d018000000000000d058000000000000d0580000000000407005000000000040700500000000000010000000000000010000000600000060461e000000000060565e0000000000'
serial_sites=None
serial_patterns=[('544f50414c5f534f555243455f53455249414c5f410a', '5561626364656667'), ('544f50414c5f534f555243455f53455249414c5f420a', '3132333435363738')]
extent_inventory={'function': 'topal_x86_systems_static_image_process_enter', 'address': 45671296, 'calls': [{'nearest_labels': ['static-process-shell-uname'], 'offset': 27139, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-getcwd'], 'offset': 27861, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-chdir-root-profile-ready'], 'offset': 28626, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-pwd-getcwd'], 'offset': 29345, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-initial-getcwd'], 'offset': 29969, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['owned-header-poll-profile-ready'], 'offset': 33496, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-header-poll-ready'], 'offset': 33583, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-query-default-common'], 'offset': 36023, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-install-quit-ignore'], 'offset': 36684, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-read-fstat-ready'], 'offset': 38450, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-refill-command-read-facts-ready'], 'offset': 39534, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-root-test-command-read-profile-ready'], 'offset': 40444, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-header-read-profile-ready'], 'offset': 41322, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-pwd-command-read-profile-ready'], 'offset': 42436, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['owned-shell-serial-copy'], 'offset': 43425, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['runtime-exec-path-byte'], 'offset': 44431, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 166, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['runtime-exec-vector-entry'], 'offset': 44792, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 189, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['runtime-exec-argument-byte'], 'offset': 44888, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 206, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-console-fstat-candidate-ready'], 'offset': 52962, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-dev-tty-open-miss-exact'], 'offset': 55270, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-profile-open'], 'offset': 56053, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-open-clean', 'static-process-shell-bin-busybox-open-profile-ready'], 'offset': 56936, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-reopen-profile-ready'], 'offset': 57843, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-root-glob-open-profile-ready'], 'offset': 58659, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-empty-bin-glob-open-profile-ready'], 'offset': 59435, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-root-glob-fstat-profile-ready'], 'offset': 60174, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-empty-bin-glob-fstat-profile-ready'], 'offset': 62337, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-profile-fstat'], 'offset': 64531, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-root-glob-getdents-first'], 'offset': 68177, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-empty-bin-glob-getdents-first'], 'offset': 70022, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-applet-newline-candidate-ready'], 'offset': 75142, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-explicit-close-write-clean', 'static-process-shell-bin-busybox-explicit-close-write-profile-ready'], 'offset': 76007, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-warning-candidate'], 'offset': 76915, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-prompt'], 'offset': 80006, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-refill-prompt'], 'offset': 81477, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-root-condition-write-clean', 'static-process-shell-root-condition-write-profile-ready'], 'offset': 82375, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-empty-bin-glob-write-clean', 'static-process-shell-empty-bin-glob-write-profile-ready'], 'offset': 83237, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-executable-write-clean', 'static-process-shell-bin-busybox-executable-write-profile-ready'], 'offset': 84283, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-nonempty-write-clean', 'static-process-shell-bin-busybox-nonempty-write-profile-ready'], 'offset': 85284, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-open-write-clean', 'static-process-shell-bin-busybox-open-write-profile-ready'], 'offset': 86299, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-header-write-profile-ready'], 'offset': 87305, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-named-root-condition-write-clean', 'static-process-shell-named-root-condition-write-profile-ready'], 'offset': 88236, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-pwd-write-profile-ready'], 'offset': 89191, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-third-prompt'], 'offset': 90027, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-fourth-prompt-profile-ready'], 'offset': 91024, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-banner'], 'offset': 92125, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-echo-candidate-ready'], 'offset': 93521, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-execve-signal-number-ready'], 'offset': 94708, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-sigchld-install'], 'offset': 95395, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-initial-process-name'], 'offset': 95961, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-stat-path'], 'offset': 98915, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-named-root-directory-stat-path'], 'offset': 99039, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-named-root-directory-stat-path-ready'], 'offset': 99332, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-root-directory-stat-profile-ready'], 'offset': 101802, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-mailpath-stat-miss'], 'offset': 102873, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-clean-config-path', 'static-process-config-path-ready'], 'offset': 103476, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-stack-limit'], 'offset': 107362, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-getrandom'], 'offset': 108204, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['runtime-console-payload', 'runtime-write-fd-ready'], 'offset': 131616, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 427, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['fd-pread-count-ready'], 'offset': 140542, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_positional_transfer.rs', 'source_line': 60, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['fd-read-count-ready'], 'offset': 141402, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_file_descriptions.rs', 'source_line': 480, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['fd-serial-count-ready'], 'offset': 142334, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_file_descriptions.rs', 'source_line': 514, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['fd-stat'], 'offset': 143474, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_file_descriptions.rs', 'source_line': 586, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['fd-limit-publish'], 'offset': 144285, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_file_descriptions.rs', 'source_line': 649, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['process-signal-action'], 'offset': 145214, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_signals.rs', 'source_line': 39, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['process-signal-action-publish'], 'offset': 145584, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_signals.rs', 'source_line': 53, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['process-signal-mask'], 'offset': 145783, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_signals.rs', 'source_line': 39, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['process-signal-mask-publish'], 'offset': 145968, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_signals.rs', 'source_line': 53, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['namespace-copy-byte'], 'offset': 151753, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_namespace_walk.rs', 'source_line': 151, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['namespace-stat-object'], 'offset': 155270, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_namespace.rs', 'source_line': 327, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['namespace-cwd-length'], 'offset': 156830, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_namespace.rs', 'source_line': 422, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['namespace-directory-copy'], 'offset': 158576, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_namespace.rs', 'source_line': 607, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['named-directory-publish'], 'offset': 161434, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_namespace_named.rs', 'source_line': 360, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['realtime-seconds'], 'offset': 177579, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_realtime.rs', 'source_line': 45, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['process-observations-uname'], 'offset': 185975, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_observations.rs', 'source_line': 60, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['process-time-gettime'], 'offset': 186810, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_time.rs', 'source_line': 76, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['process-time-getres'], 'offset': 186999, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_time.rs', 'source_line': 76, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['process-time-copy-interval'], 'offset': 187213, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_time.rs', 'source_line': 139, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['child-thread-local'], 'offset': 190244, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_children.rs', 'source_line': 136, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['child-clear-registration-ready'], 'offset': 194631, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_children.rs', 'source_line': 316, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['child-wait-matched'], 'offset': 196557, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_children.rs', 'source_line': 413, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['pipe-descriptions-ready'], 'offset': 203337, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_pipe_create.rs', 'source_line': 109, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['pipe-read-count-ready'], 'offset': 204492, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_pipe_transfer.rs', 'source_line': 56, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['pipe-write-count-ready'], 'offset': 205322, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_pipe_transfer.rs', 'source_line': 131, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['pipe-ioctl'], 'offset': 207832, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_pipes.rs', 'source_line': 152, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['file-read-end'], 'offset': 221407, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_volatile_transfer.rs', 'source_line': 59, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['file-pread-end'], 'offset': 222189, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_volatile_transfer.rs', 'source_line': 59, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['file-write-end'], 'offset': 222985, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_volatile_transfer.rs', 'source_line': 201, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['file-pwrite-end'], 'offset': 224056, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_volatile_transfer.rs', 'source_line': 201, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['runtime-environment-entry'], 'offset': 231462, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_exec_environment.rs', 'source_line': 25, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['runtime-environment-byte'], 'offset': 231558, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_exec_environment.rs', 'source_line': 41, 'target': 'runtime-readable-extent'}], 'callback_bindings': {'passed': True, 'kernel_sha256': '04f4f0b01552e5d724187ae841bec2844d00d113eee2ee940adb8304997433c6', 'root_sha256': '642b4b71a51bbeefa92e77dae544a1355bf41b0520f9f18461ef5fbc3c962ea8', 'process_graph_sha256': 'afdacb7c3705041ec40dac4f9f6ab45fb58a62a64afe468c4c71d91768a4da32', 'baseline_provenance_sha256': '09ea31cc33350cee2a108db6688a09d85510c7c1d6a72065859016c67791f032', 'root_call_offset': 60704, 'root_stack_pushes': 13, 'callbacks': [{'gs_slot': 952, 'argument': 13, 'entry_stack_offset': 56, 'native_export': 'topal_kernel_file_open_policy_v1', 'target': 46270368, 'sections': [{'marker': 'topal.policy-inputs.kernel-library.0', 'object_section': 2, 'address': 46267072, 'bytes': 3517, 'executable': True, 'object_sha256': '36515d3ed402aeb3f0adb8c2ba945c235d8eed14e480488113d1851e2b7ea533', 'linked_sha256': '1d1ea8cd441dbe2df1a8c8add5b9cf2e614f3df4f1df7b3e26296d321e6a31ce'}, {'marker': 'topal.policy-inputs.kernel-library.1', 'object_section': 6, 'address': 44999224, 'bytes': 64, 'executable': False, 'object_sha256': 'f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b', 'linked_sha256': '1192efe08c868adb3f976532784e6ac786c4be58dafebfeb68bfc1c0a9f9e8f1'}]}, {'gs_slot': 960, 'argument': 14, 'entry_stack_offset': 64, 'native_export': 'topal_kernel_sleep_deadline_policy_v1', 'target': 46240896, 'sections': [{'marker': 'topal.policy-inputs.time-library.0', 'object_section': 3, 'address': 46235568, 'bytes': 73, 'executable': True, 'object_sha256': 'cb9c496ef5234e5129bb5b82975701cdc84ec32909a090bae9dd90fe1a7490e4', 'linked_sha256': 'cb9c496ef5234e5129bb5b82975701cdc84ec32909a090bae9dd90fe1a7490e4'}, {'marker': 'topal.policy-inputs.time-library.1', 'object_section': 6, 'address': 46235648, 'bytes': 43, 'executable': True, 'object_sha256': 'a5ad2f00aa57a07d946bc0db327b8340f24835675843526a0834b73409e72a55', 'linked_sha256': '2a1b84e4bc104f92c8c52826ad3447bfb8ddacc334cf009a45096b2af7d73e1f'}, {'marker': 'topal.policy-inputs.time-library.2', 'object_section': 10, 'address': 46235696, 'bytes': 31, 'executable': True, 'object_sha256': '0714afaaaf62d1196e5bea3b2380215dd79014614487563278fe5cae9dca533d', 'linked_sha256': '9259263f0ea25538245bcc7bd145ad321177643f4dde53b869a9bdff0cac6462'}, {'marker': 'topal.policy-inputs.time-library.3', 'object_section': 14, 'address': 46235728, 'bytes': 48, 'executable': True, 'object_sha256': '3b4b9bbf2d1192bd245b821b52415a3203fe5b725ccc709a2dce83e51a4bd865', 'linked_sha256': 'b7765d427c0ab33ac2f5e19fde4bdd111f726a689f6090d8249c3f9f019ca48b'}, {'marker': 'topal.policy-inputs.time-library.4', 'object_section': 18, 'address': 46235776, 'bytes': 10, 'executable': True, 'object_sha256': 'e65d0e5b3e004bca7221190010f8d17cf9cccee4894c85ad38b21fc472e35095', 'linked_sha256': 'e65d0e5b3e004bca7221190010f8d17cf9cccee4894c85ad38b21fc472e35095'}, {'marker': 'topal.policy-inputs.time-library.5', 'object_section': 21, 'address': 46235792, 'bytes': 79, 'executable': True, 'object_sha256': '872017bf481b6c1c97279bcc4acfe82207572829807084c4c2f4dbb1c363ac44', 'linked_sha256': '2aa435852bd643649896da02c35845b4b0b87f570b0f00c3569646b9ffed83df'}, {'marker': 'topal.policy-inputs.time-library.6', 'object_section': 25, 'address': 46235872, 'bytes': 127, 'executable': True, 'object_sha256': '767de80f9cb4df09613ff8c7490087362ebee98e020ec3fb650a2d8943340549', 'linked_sha256': '2aa9618f8002e8236571352027b95215abc3d3d0a037f0af9105e7c9a336c271'}, {'marker': 'topal.policy-inputs.time-library.7', 'object_section': 29, 'address': 46236000, 'bytes': 48, 'executable': True, 'object_sha256': 'b57c05d1d63a3571aa80996828bc652cf377b32827724be3b85363ad5fea8917', 'linked_sha256': 'b57c05d1d63a3571aa80996828bc652cf377b32827724be3b85363ad5fea8917'}, {'marker': 'topal.policy-inputs.time-library.8', 'object_section': 32, 'address': 46236048, 'bytes': 127, 'executable': True, 'object_sha256': '8207954e261e2daab0c96d6e91fdb6a8df841b45e00ca412b5e267bf5d98df4d', 'linked_sha256': '8207954e261e2daab0c96d6e91fdb6a8df841b45e00ca412b5e267bf5d98df4d'}, {'marker': 'topal.policy-inputs.time-library.9', 'object_section': 35, 'address': 46236176, 'bytes': 116, 'executable': True, 'object_sha256': '89037fa8911cb742fd2e36b2eb95f1a93cf8c5b5e1f5468ce900e97d9af061fd', 'linked_sha256': '8bf2da28b4778e11ce762d1740ef4def857d461d3f34157641a22863d29ee0c8'}, {'marker': 'topal.policy-inputs.time-library.10', 'object_section': 39, 'address': 46236304, 'bytes': 96, 'executable': True, 'object_sha256': 'b7f19efda9a6cc06a496a869ea51181d095beb8c2cbd6d650485302834f2c1ed', 'linked_sha256': 'e4b84df989052f0aaca02f2171d307270602e37cf5dc3a9c98c43f1914a1d121'}, {'marker': 'topal.policy-inputs.time-library.11', 'object_section': 43, 'address': 46236400, 'bytes': 158, 'executable': True, 'object_sha256': 'f1682ed576c8f19f011cfdca0f8ddf298a7d002fb9a27c4679050fde11d5bc6b', 'linked_sha256': 'f1682ed576c8f19f011cfdca0f8ddf298a7d002fb9a27c4679050fde11d5bc6b'}, {'marker': 'topal.policy-inputs.time-library.12', 'object_section': 46, 'address': 46236560, 'bytes': 212, 'executable': True, 'object_sha256': '569a1b2edad0d2efc6b1be5b07b7f3a5384cf2b6ecc0ee81aeac92171222fe3f', 'linked_sha256': 'd5180bd7440331a2303f0d4ac673b9517ee66fba8cb7e02d59f4bab0b6f5ef86'}, {'marker': 'topal.policy-inputs.time-library.13', 'object_section': 50, 'address': 46236784, 'bytes': 239, 'executable': True, 'object_sha256': '8ee086baa73317c5efd1aa880e03d23b9ff09d07184904a3ba9dfd4a25d12a43', 'linked_sha256': '2bd94405a08513199832a40eed0e325b2b604d87d11b1bd2d6016160e71512e7'}, {'marker': 'topal.policy-inputs.time-library.14', 'object_section': 54, 'address': 46237024, 'bytes': 198, 'executable': True, 'object_sha256': '040afd43269a673f1ca481e0806d771988188c46eefdc6c439537f822ca87287', 'linked_sha256': '43c3f31207f44386c3af2b0465188c79e1aed4f4495dc8a54f78fc41c81a48fb'}, {'marker': 'topal.policy-inputs.time-library.15', 'object_section': 58, 'address': 46237232, 'bytes': 339, 'executable': True, 'object_sha256': 'c6b07ea70d78afdd3f3514c5fab78ed1472067cd30f842a78b33dbae6f7197a6', 'linked_sha256': '32aeb429fefe6058a40f92e61f66003839e2c276fe0eb6b22a40ad5f90194a8f'}, {'marker': 'topal.policy-inputs.time-library.16', 'object_section': 62, 'address': 46237584, 'bytes': 538, 'executable': True, 'object_sha256': '031c16e7e9fe6eb82bfec5dd4b94963612325082ab2f96634a26d901a2373e9c', 'linked_sha256': '3efc1cbe1c64a8eabf284dfe2bdf05475b7ee430956ac80b19d47e87f12b7e41'}, {'marker': 'topal.policy-inputs.time-library.17', 'object_section': 66, 'address': 46238128, 'bytes': 28, 'executable': True, 'object_sha256': 'd11865a7a2e5c535d1846140ea69e97c0d55ff666cedd2532cfc339bb73cad94', 'linked_sha256': 'd11865a7a2e5c535d1846140ea69e97c0d55ff666cedd2532cfc339bb73cad94'}, {'marker': 'topal.policy-inputs.time-library.18', 'object_section': 69, 'address': 46238160, 'bytes': 47, 'executable': True, 'object_sha256': '8879e5795dc0326415b3925d1f0c210d24ea80121d0c0f13da18572f54f9b508', 'linked_sha256': '670155cc82e36020d862b9366a151dc96529f95f978e8bf96dd46ea78b77737f'}, {'marker': 'topal.policy-inputs.time-library.19', 'object_section': 73, 'address': 46238208, 'bytes': 9, 'executable': True, 'object_sha256': 'b1692178a17fd71398c4c4c05612bcddec0991e1ba16e887345afc6c929106fd', 'linked_sha256': 'b1692178a17fd71398c4c4c05612bcddec0991e1ba16e887345afc6c929106fd'}, {'marker': 'topal.policy-inputs.time-library.20', 'object_section': 76, 'address': 46238224, 'bytes': 10, 'executable': True, 'object_sha256': 'e65d0e5b3e004bca7221190010f8d17cf9cccee4894c85ad38b21fc472e35095', 'linked_sha256': 'e65d0e5b3e004bca7221190010f8d17cf9cccee4894c85ad38b21fc472e35095'}, {'marker': 'topal.policy-inputs.time-library.21', 'object_section': 79, 'address': 46238240, 'bytes': 112, 'executable': True, 'object_sha256': '5b476887ad6b39b421a6fe01159be4c83f4575445f1c70d53044e754763b2999', 'linked_sha256': '1052c44ab09ff99a1d138aa8cc56f41a53db6c642fa432216d51d03bfcca00dd'}, {'marker': 'topal.policy-inputs.time-library.22', 'object_section': 83, 'address': 46238352, 'bytes': 25, 'executable': True, 'object_sha256': 'abef1dd9c8df305345b7760ea0fbd491bd0aab56bb629e69328e9ebcfa95a427', 'linked_sha256': 'abef1dd9c8df305345b7760ea0fbd491bd0aab56bb629e69328e9ebcfa95a427'}, {'marker': 'topal.policy-inputs.time-library.23', 'object_section': 86, 'address': 46238384, 'bytes': 422, 'executable': True, 'object_sha256': '954013d9499f9a603a14d29d2fca11332ac431cda80acc9db210d18d22433746', 'linked_sha256': '3dc9647aafc5ee01903157daa95bad3dca7404951a9f31eaa8e66297d616abc1'}, {'marker': 'topal.policy-inputs.time-library.24', 'object_section': 90, 'address': 46238816, 'bytes': 288, 'executable': True, 'object_sha256': '54a1330cbe0e5bdead5dd8da3311ad85409a2204667c83d0da65aeafd55d2df2', 'linked_sha256': '529b70b9485877b267a07fe2b42445e0a091b161c956f634e34f15994ce0efdd'}, {'marker': 'topal.policy-inputs.time-library.25', 'object_section': 94, 'address': 46239104, 'bytes': 78, 'executable': True, 'object_sha256': '6a3f5ae2fe2de70e3b0162e4327d420908e7a1ea6a3b79fd9e67586e9098af6a', 'linked_sha256': '6a3f5ae2fe2de70e3b0162e4327d420908e7a1ea6a3b79fd9e67586e9098af6a'}, {'marker': 'topal.policy-inputs.time-library.26', 'object_section': 97, 'address': 46239184, 'bytes': 52, 'executable': True, 'object_sha256': 'c1108216f2d57f3b6165da46e0fe768d6cc326826e5c32c73079a5b5501dc750', 'linked_sha256': '3d18c9146e191fc05c7f4927758b4ab7e8e7ba7ea49f7943b3f3109a44823842'}, {'marker': 'topal.policy-inputs.time-library.27', 'object_section': 101, 'address': 46239248, 'bytes': 119, 'executable': True, 'object_sha256': 'cdafa3474e97832b8e4e45936c1d652467ea5784c96c6578a1908a72ad02bd96', 'linked_sha256': '343fee65ecb3b79f12f29809957d8171b8150e0333e6e7fdb37ce874ec698183'}, {'marker': 'topal.policy-inputs.time-library.28', 'object_section': 105, 'address': 46239376, 'bytes': 11, 'executable': True, 'object_sha256': 'd6cce27151649426900104a94856db51bea1ccc7fe263bfc350f8ed8268d885a', 'linked_sha256': '0d9a3c30bcd0e014adf8a1705ac6b7087067196b44e797b6e40245fe7ff1a6d2'}, {'marker': 'topal.policy-inputs.time-library.29', 'object_section': 109, 'address': 46239392, 'bytes': 9, 'executable': True, 'object_sha256': '7fc921d45c8e0dac7d2aa42f26536c8830cd35bfc8bce127eab50a452af67b52', 'linked_sha256': 'ad7a1faf3b3487c046cb727fef26deef92638f84aff7270f73a1659c54a96afb'}, {'marker': 'topal.policy-inputs.time-library.30', 'object_section': 113, 'address': 46239408, 'bytes': 80, 'executable': True, 'object_sha256': 'e3135068013ae79169c763a681e62f163f67fbe2f920dcf77298ca05dcc9e40c', 'linked_sha256': 'fbbdea28b1837c91e31d216c1c55b6493d87b50a4d31e02163406f40681478c7'}, {'marker': 'topal.policy-inputs.time-library.31', 'object_section': 117, 'address': 46239488, 'bytes': 81, 'executable': True, 'object_sha256': '84ead12f4221d96701c41e7d72dd59bcb6aed0e758ed2da549e6d11b5edceb9c', 'linked_sha256': 'c58fdcbd31be60d5330000287fca819cc9f2f674ff9a5331d5bf12995c22def6'}, {'marker': 'topal.policy-inputs.time-library.32', 'object_section': 121, 'address': 46239584, 'bytes': 123, 'executable': True, 'object_sha256': '6288cc443b180529adb209bb5892fcb88480cbab36d2cd3f14c48b695ce163b2', 'linked_sha256': '0bdaede1f0672e74ff45c9cd3096b3d5865760a42772180117a03e2c30a01724'}, {'marker': 'topal.policy-inputs.time-library.33', 'object_section': 125, 'address': 46239712, 'bytes': 116, 'executable': True, 'object_sha256': '3864a191e80b6d57cb3b6cca1ae7e07507ca0c0b168b53385a9c5716e111ee1a', 'linked_sha256': 'dcd0a8fb098d46e901eb956ddd2d02257859aa2ac9e5401091d736ea4659a836'}, {'marker': 'topal.policy-inputs.time-library.34', 'object_section': 129, 'address': 46239840, 'bytes': 72, 'executable': True, 'object_sha256': 'cc0441c3e9615f65b65a48159490141ec7a361c6e171bda8dd64210bd30647ff', 'linked_sha256': '1cad56fe2d9ad5a6c9e174c1baf5577906683dc65e237346d52615738711a7f1'}, {'marker': 'topal.policy-inputs.time-library.35', 'object_section': 133, 'address': 46239920, 'bytes': 25, 'executable': True, 'object_sha256': '677b1c6e62b27f407c9ed53b3c38f49e3b53ecab9b664f3a47b9d2abdd5321e3', 'linked_sha256': '677b1c6e62b27f407c9ed53b3c38f49e3b53ecab9b664f3a47b9d2abdd5321e3'}, {'marker': 'topal.policy-inputs.time-library.36', 'object_section': 136, 'address': 46239952, 'bytes': 150, 'executable': True, 'object_sha256': '98e4cabdc9f7e54b405acd3715cb5eabbd7db835e526325de584bdb94546cfbe', 'linked_sha256': '35395813481e18e5be03333bd919b09b09f240ead2a9cacbfcc41c84622cb4e7'}, {'marker': 'topal.policy-inputs.time-library.37', 'object_section': 140, 'address': 46240112, 'bytes': 281, 'executable': True, 'object_sha256': '14751080971459cc1ca083bb3c7604ee3554abecc26cda47b1a6ce8198c36f86', 'linked_sha256': 'a36144376f54ba4facffcc2cd17ba76d3678e9640da0bd47f650c7fee1449b19'}, {'marker': 'topal.policy-inputs.time-library.38', 'object_section': 144, 'address': 46240400, 'bytes': 319, 'executable': True, 'object_sha256': '550c25bee07e35d506faaa963690e1642f5ec616ad5de4aa2d546e68ae057593', 'linked_sha256': 'db7497c9a6a3b4df9641279303c347abcb3acef43fe89c1d16ad525643e50999'}, {'marker': 'topal.policy-inputs.time-library.39', 'object_section': 148, 'address': 46240720, 'bytes': 162, 'executable': True, 'object_sha256': 'a135783bee8b15efb2098568b30e85e381ef18abccfff60d2f097142364acf7d', 'linked_sha256': 'f9c1ee2027ab854a82a7ecc79e2badba447aed17c0938cc28bd3c491a7828258'}, {'marker': 'topal.policy-inputs.time-library.40', 'object_section': 152, 'address': 46240896, 'bytes': 272, 'executable': True, 'object_sha256': 'dcade463769a07783db3c2ddd705b3458ad337fdcf39c8667af46b3fb8dbfc94', 'linked_sha256': '150a31dc64700df9328335d7c7491cee4f9869154323dbebfdb7383c61621014'}, {'marker': 'topal.policy-inputs.time-library.41', 'object_section': 156, 'address': 44993976, 'bytes': 264, 'executable': False, 'object_sha256': 'c4dbfbb1e49e81e12e5538f195f08a188d8e75b76d77044e0158969fbc3b9ee3', 'linked_sha256': 'c4dbfbb1e49e81e12e5538f195f08a188d8e75b76d77044e0158969fbc3b9ee3'}, {'marker': 'topal.policy-inputs.time-library.42', 'object_section': 157, 'address': 44994240, 'bytes': 32, 'executable': False, 'object_sha256': '66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925', 'linked_sha256': '66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925'}]}, {'gs_slot': 976, 'argument': 16, 'entry_stack_offset': 80, 'native_export': 'topal_kernel_static_elf64_header_policy_v1', 'target': 46248080, 'sections': [{'marker': 'topal.policy-inputs.image-library.0', 'object_section': 3, 'address': 46241168, 'bytes': 73, 'executable': True, 'object_sha256': 'cb9c496ef5234e5129bb5b82975701cdc84ec32909a090bae9dd90fe1a7490e4', 'linked_sha256': 'cb9c496ef5234e5129bb5b82975701cdc84ec32909a090bae9dd90fe1a7490e4'}, {'marker': 'topal.policy-inputs.image-library.1', 'object_section': 6, 'address': 46241248, 'bytes': 43, 'executable': True, 'object_sha256': 'a5ad2f00aa57a07d946bc0db327b8340f24835675843526a0834b73409e72a55', 'linked_sha256': '3325445fe3f2e8a777a27673246092b542a801530ce1cfff5c277fd71573ed7a'}, {'marker': 'topal.policy-inputs.image-library.2', 'object_section': 10, 'address': 46241296, 'bytes': 31, 'executable': True, 'object_sha256': '0714afaaaf62d1196e5bea3b2380215dd79014614487563278fe5cae9dca533d', 'linked_sha256': '6166d221d7aaf2b62164d45a79939f4c2665eea3a82dfbce325837b830ea70c3'}, {'marker': 'topal.policy-inputs.image-library.3', 'object_section': 14, 'address': 46241328, 'bytes': 48, 'executable': True, 'object_sha256': '3b4b9bbf2d1192bd245b821b52415a3203fe5b725ccc709a2dce83e51a4bd865', 'linked_sha256': '441ca8fb9ed92045c6d694da15658b07b1ee84d59dab2eae48f770d4fed829ed'}, {'marker': 'topal.policy-inputs.image-library.4', 'object_section': 18, 'address': 46241376, 'bytes': 10, 'executable': True, 'object_sha256': 'e65d0e5b3e004bca7221190010f8d17cf9cccee4894c85ad38b21fc472e35095', 'linked_sha256': 'e65d0e5b3e004bca7221190010f8d17cf9cccee4894c85ad38b21fc472e35095'}, {'marker': 'topal.policy-inputs.image-library.5', 'object_section': 21, 'address': 46241392, 'bytes': 79, 'executable': True, 'object_sha256': '872017bf481b6c1c97279bcc4acfe82207572829807084c4c2f4dbb1c363ac44', 'linked_sha256': 'a427c1e578f66bd7868d00e4c293a185dea289ab9fa361566c8551a924b40ed2'}, {'marker': 'topal.policy-inputs.image-library.6', 'object_section': 25, 'address': 46241472, 'bytes': 127, 'executable': True, 'object_sha256': '767de80f9cb4df09613ff8c7490087362ebee98e020ec3fb650a2d8943340549', 'linked_sha256': '7497439f6e9915a379b0f3e0c3c638387f8967d79a209686441640a78f6f13fc'}, {'marker': 'topal.policy-inputs.image-library.7', 'object_section': 29, 'address': 46241600, 'bytes': 48, 'executable': True, 'object_sha256': 'b57c05d1d63a3571aa80996828bc652cf377b32827724be3b85363ad5fea8917', 'linked_sha256': 'b57c05d1d63a3571aa80996828bc652cf377b32827724be3b85363ad5fea8917'}, {'marker': 'topal.policy-inputs.image-library.8', 'object_section': 32, 'address': 46241648, 'bytes': 127, 'executable': True, 'object_sha256': '8207954e261e2daab0c96d6e91fdb6a8df841b45e00ca412b5e267bf5d98df4d', 'linked_sha256': '8207954e261e2daab0c96d6e91fdb6a8df841b45e00ca412b5e267bf5d98df4d'}, {'marker': 'topal.policy-inputs.image-library.9', 'object_section': 35, 'address': 46241776, 'bytes': 158, 'executable': True, 'object_sha256': 'f1682ed576c8f19f011cfdca0f8ddf298a7d002fb9a27c4679050fde11d5bc6b', 'linked_sha256': 'f1682ed576c8f19f011cfdca0f8ddf298a7d002fb9a27c4679050fde11d5bc6b'}, {'marker': 'topal.policy-inputs.image-library.10', 'object_section': 38, 'address': 46241936, 'bytes': 212, 'executable': True, 'object_sha256': '569a1b2edad0d2efc6b1be5b07b7f3a5384cf2b6ecc0ee81aeac92171222fe3f', 'linked_sha256': 'd5180bd7440331a2303f0d4ac673b9517ee66fba8cb7e02d59f4bab0b6f5ef86'}, {'marker': 'topal.policy-inputs.image-library.11', 'object_section': 42, 'address': 46242160, 'bytes': 239, 'executable': True, 'object_sha256': '8ee086baa73317c5efd1aa880e03d23b9ff09d07184904a3ba9dfd4a25d12a43', 'linked_sha256': 'e4a81ab23adac0518f9602b0174fca663c0a4136b21681703d544ff71536d530'}, {'marker': 'topal.policy-inputs.image-library.12', 'object_section': 46, 'address': 46242400, 'bytes': 198, 'executable': True, 'object_sha256': '040afd43269a673f1ca481e0806d771988188c46eefdc6c439537f822ca87287', 'linked_sha256': '2b6f1cab4fa8ea8dc97c8cdf5dda673e0d8e43e663c3f826fa987a47dd8d0daf'}, {'marker': 'topal.policy-inputs.image-library.13', 'object_section': 50, 'address': 46242608, 'bytes': 339, 'executable': True, 'object_sha256': 'c6b07ea70d78afdd3f3514c5fab78ed1472067cd30f842a78b33dbae6f7197a6', 'linked_sha256': 'b8cd68439abf4d12d1b795818281dfae2ca26f85d129fa35ff2bfd55863dceaf'}, {'marker': 'topal.policy-inputs.image-library.14', 'object_section': 54, 'address': 46242960, 'bytes': 538, 'executable': True, 'object_sha256': '031c16e7e9fe6eb82bfec5dd4b94963612325082ab2f96634a26d901a2373e9c', 'linked_sha256': 'c6faee03d698cf1b2e750834c061b50a54a9fad66c258402c5bff003c35a71ed'}, {'marker': 'topal.policy-inputs.image-library.15', 'object_section': 58, 'address': 46243504, 'bytes': 28, 'executable': True, 'object_sha256': 'd11865a7a2e5c535d1846140ea69e97c0d55ff666cedd2532cfc339bb73cad94', 'linked_sha256': 'd11865a7a2e5c535d1846140ea69e97c0d55ff666cedd2532cfc339bb73cad94'}, {'marker': 'topal.policy-inputs.image-library.16', 'object_section': 61, 'address': 46243536, 'bytes': 78, 'executable': True, 'object_sha256': '6a3f5ae2fe2de70e3b0162e4327d420908e7a1ea6a3b79fd9e67586e9098af6a', 'linked_sha256': '6a3f5ae2fe2de70e3b0162e4327d420908e7a1ea6a3b79fd9e67586e9098af6a'}, {'marker': 'topal.policy-inputs.image-library.17', 'object_section': 64, 'address': 46243616, 'bytes': 52, 'executable': True, 'object_sha256': 'c1108216f2d57f3b6165da46e0fe768d6cc326826e5c32c73079a5b5501dc750', 'linked_sha256': '3d18c9146e191fc05c7f4927758b4ab7e8e7ba7ea49f7943b3f3109a44823842'}, {'marker': 'topal.policy-inputs.image-library.18', 'object_section': 68, 'address': 46243680, 'bytes': 119, 'executable': True, 'object_sha256': 'cdafa3474e97832b8e4e45936c1d652467ea5784c96c6578a1908a72ad02bd96', 'linked_sha256': 'd6664ad963861c77a32cd56c49b1afe0d8389c2bf9fc156af79ca53372906776'}, {'marker': 'topal.policy-inputs.image-library.19', 'object_section': 72, 'address': 46243808, 'bytes': 11, 'executable': True, 'object_sha256': 'd6cce27151649426900104a94856db51bea1ccc7fe263bfc350f8ed8268d885a', 'linked_sha256': '9fc573012937f8a5f92626a7b7331daff28ea4dc0817937317de3b7f635596d0'}, {'marker': 'topal.policy-inputs.image-library.20', 'object_section': 76, 'address': 46243824, 'bytes': 9, 'executable': True, 'object_sha256': '7fc921d45c8e0dac7d2aa42f26536c8830cd35bfc8bce127eab50a452af67b52', 'linked_sha256': 'e80fbf0f8290e931be5153b75582339100f82223e803c0b612beb9a966712271'}, {'marker': 'topal.policy-inputs.image-library.21', 'object_section': 80, 'address': 46243840, 'bytes': 80, 'executable': True, 'object_sha256': 'e3135068013ae79169c763a681e62f163f67fbe2f920dcf77298ca05dcc9e40c', 'linked_sha256': 'ab320ead517083d042cc8f645d2baba5fa927966c309d550c02cdeb44100e7a1'}, {'marker': 'topal.policy-inputs.image-library.22', 'object_section': 84, 'address': 46243920, 'bytes': 81, 'executable': True, 'object_sha256': '84ead12f4221d96701c41e7d72dd59bcb6aed0e758ed2da549e6d11b5edceb9c', 'linked_sha256': '73d4e88dc340e8ac22aa8a43af7397425dd72f0f1f3b8daa66855008b4dd1658'}, {'marker': 'topal.policy-inputs.image-library.23', 'object_section': 88, 'address': 46244016, 'bytes': 81, 'executable': True, 'object_sha256': 'bf66244d980fc308e4ad8864f7e54167f1349fce4d1536788c33c67950834286', 'linked_sha256': 'c1dfc3fa4b6cc91f956ac146a08b5f13fdd2ceb9289ea230d522d7b1737c9b8d'}, {'marker': 'topal.policy-inputs.image-library.24', 'object_section': 92, 'address': 46244112, 'bytes': 81, 'executable': True, 'object_sha256': 'fab9e6998e03fb871e4fa5659f4b074e769277ff52ac6bf7a9107a07df97c0b2', 'linked_sha256': '3b33f3a99399a03cad7e8e802502b0c976554e38735a04c53e824f249cb91a29'}, {'marker': 'topal.policy-inputs.image-library.25', 'object_section': 96, 'address': 46244208, 'bytes': 150, 'executable': True, 'object_sha256': '36b8f85a7f794e5d1300394417af465c1dd75f0801cff833f5f52ef1c3e86555', 'linked_sha256': '8c78f9a273ad4cd22468709421f279bde04551ae7b29ec45e22059e585ae0ce6'}, {'marker': 'topal.policy-inputs.image-library.26', 'object_section': 100, 'address': 46244368, 'bytes': 86, 'executable': True, 'object_sha256': '2a38710e5eaccf2fad6c31fdc4f04574fd4d331cc3dda9a411fa2b288045d601', 'linked_sha256': '4b7d070b47e6365652df7703a46110b25d6afc2715d0ae98b215ab9a7114654d'}, {'marker': 'topal.policy-inputs.image-library.27', 'object_section': 104, 'address': 46244464, 'bytes': 81, 'executable': True, 'object_sha256': '7440f8b9eaf6557739ad235120651539bfef4c94a7b55899e086107604892a68', 'linked_sha256': '2f7266b3ce503f9d96132d212a265fb20b43d0446ab1ccbe8c608b6f57c3ac36'}, {'marker': 'topal.policy-inputs.image-library.28', 'object_section': 108, 'address': 46244560, 'bytes': 81, 'executable': True, 'object_sha256': '443dbfd980db1f639abe9df928959ebb54d01aca77b8e9ae327cbcaf310bd6d7', 'linked_sha256': '05c3ad39223864e968aa7c90ae07b6294ed01efd479457dba92f38e4e8f68c64'}, {'marker': 'topal.policy-inputs.image-library.29', 'object_section': 112, 'address': 46244656, 'bytes': 81, 'executable': True, 'object_sha256': 'c4551145e481198310406fa38106a490baf20697f7d114538645343f77e358d8', 'linked_sha256': 'd3211030447cbad3f6358ee51fe3fd0dec7e224ca4153225a682e4e61db64d3f'}, {'marker': 'topal.policy-inputs.image-library.30', 'object_section': 116, 'address': 46244752, 'bytes': 107, 'executable': True, 'object_sha256': 'eae4103ce612db87a79677c424283caa99bfbc3025df2b2f85588eaba7850b67', 'linked_sha256': 'a1a7e677357dbe689291e64ef4ed6efb15f125b8438d5b639d78938c88a1190a'}, {'marker': 'topal.policy-inputs.image-library.31', 'object_section': 120, 'address': 46244864, 'bytes': 69, 'executable': True, 'object_sha256': 'c6821dc6c780767320170adf541891c1b89f486f44e8a1e6fc19fc06f8a69119', 'linked_sha256': 'fc32a5285e4e54017dbe1f16e05236e8a5dd639c7fab4160073f9edb76d321be'}, {'marker': 'topal.policy-inputs.image-library.32', 'object_section': 124, 'address': 46244944, 'bytes': 81, 'executable': True, 'object_sha256': 'b9e85f8cbe150b99b120fca494df263f183bab2bbe94af0a20e357eb17505e7a', 'linked_sha256': '7ffb3def495b94fe6475f70cdb00a0afa5e0e714eb5a473f7db7e21b3443623f'}, {'marker': 'topal.policy-inputs.image-library.33', 'object_section': 128, 'address': 46245040, 'bytes': 81, 'executable': True, 'object_sha256': '68703e11cc73260c8b4f953dc9ac31d6293f8604be09e17dcbd5b62376a7f946', 'linked_sha256': '5674d9377fa63cd95fc414f918372cae63ca9d224fa4d143609eac8edb6c3275'}, {'marker': 'topal.policy-inputs.image-library.34', 'object_section': 132, 'address': 46245136, 'bytes': 64, 'executable': True, 'object_sha256': 'd55e8e427d281eb4fc3fc10c38a49b3e216c8ecd795c361180a9ec5bceac73ed', 'linked_sha256': '6b67d75df1b3712ba5596624654b2997dacf85944bf8cca46f74bd0b87a50ca3'}, {'marker': 'topal.policy-inputs.image-library.35', 'object_section': 136, 'address': 46245200, 'bytes': 52, 'executable': True, 'object_sha256': 'f2cda79ae4292f910af93b0dcf3fe562bd3b4cca0075abf69abe1b861bbe4d7c', 'linked_sha256': '015eb6b601b018319341ebd1ba105125ea066bf347d2e763b0663a95d763a273'}, {'marker': 'topal.policy-inputs.image-library.36', 'object_section': 140, 'address': 46245264, 'bytes': 81, 'executable': True, 'object_sha256': '6db235e86b4128d7a977f57fe212668c9bb02744a563d22e400328b2be93cc4d', 'linked_sha256': '06f1303d25892a3e08af3df1c85ada0ea725698b76183c6da5f6a3c2f9b4e802'}, {'marker': 'topal.policy-inputs.image-library.37', 'object_section': 144, 'address': 46245360, 'bytes': 81, 'executable': True, 'object_sha256': '003f4728c4d342d6661ac3246f8cdf3645c32dd86372de5af921b90912517ca0', 'linked_sha256': '02450b2d09e1176caabed4c6e5622b451915697fd1260a85ee64ceda95e41d9e'}, {'marker': 'topal.policy-inputs.image-library.38', 'object_section': 148, 'address': 46245456, 'bytes': 64, 'executable': True, 'object_sha256': 'd55e8e427d281eb4fc3fc10c38a49b3e216c8ecd795c361180a9ec5bceac73ed', 'linked_sha256': '6bccc781740d7ab3cfb92ea31485d26ecab36e99380e11501ea7b22b3e3ac2dd'}, {'marker': 'topal.policy-inputs.image-library.39', 'object_section': 152, 'address': 46245520, 'bytes': 52, 'executable': True, 'object_sha256': 'f2cda79ae4292f910af93b0dcf3fe562bd3b4cca0075abf69abe1b861bbe4d7c', 'linked_sha256': '015eb6b601b018319341ebd1ba105125ea066bf347d2e763b0663a95d763a273'}, {'marker': 'topal.policy-inputs.image-library.40', 'object_section': 156, 'address': 46245584, 'bytes': 81, 'executable': True, 'object_sha256': 'e44aec922ab87b172538cb424e1a5cab9dca81de66801e50a9c1b0722977bcc9', 'linked_sha256': '2647bb20f0d132b6bdfee55f99268817bd9b862955e6a5fb91b8c0d365ecc653'}, {'marker': 'topal.policy-inputs.image-library.41', 'object_section': 160, 'address': 46245680, 'bytes': 81, 'executable': True, 'object_sha256': 'd28e8d0614ebb8317cc7e34365e1a68f13aa86173ba03ef22f07439695ba8ea9', 'linked_sha256': 'e1813e9c2e0042cd84295d0c71ee5c6fd14719d66e7a06ab83b427234a04c5de'}, {'marker': 'topal.policy-inputs.image-library.42', 'object_section': 164, 'address': 46245776, 'bytes': 81, 'executable': True, 'object_sha256': 'fdccc6ccb12e7e58356b67ed6394e59c2ebb784302be9881d1387aaaad8dddc9', 'linked_sha256': '52584cd23b6797c27d6fd20ebe3675c7d8f482204fb3c1b9063202a9976d3f90'}, {'marker': 'topal.policy-inputs.image-library.43', 'object_section': 168, 'address': 46245872, 'bytes': 81, 'executable': True, 'object_sha256': '4e8711ffaed8385ec05610d1d66e06ff7c8f98def5e4105a7f239d35af9a70f4', 'linked_sha256': '39e5076c0795dd5213bba0484508132e7139fb6d80ccafea5dea9ff41b30946e'}, {'marker': 'topal.policy-inputs.image-library.44', 'object_section': 172, 'address': 46245968, 'bytes': 150, 'executable': True, 'object_sha256': '36b8f85a7f794e5d1300394417af465c1dd75f0801cff833f5f52ef1c3e86555', 'linked_sha256': '6c921aa0933d98fe8d0b4ae4cf38ee2d5d9c8d31a9acad7b3f29f78645e6e5e4'}, {'marker': 'topal.policy-inputs.image-library.45', 'object_section': 176, 'address': 46246128, 'bytes': 86, 'executable': True, 'object_sha256': '2a38710e5eaccf2fad6c31fdc4f04574fd4d331cc3dda9a411fa2b288045d601', 'linked_sha256': 'd2cfa3b89b10253f08bdc8410e71a46d0f47bdea110251b58291d82ef2af3b68'}, {'marker': 'topal.policy-inputs.image-library.46', 'object_section': 180, 'address': 46246224, 'bytes': 81, 'executable': True, 'object_sha256': 'ec656a5e094fe92d1954d1de711f867ee4590d08d2575490c34158b6f69b471d', 'linked_sha256': 'd88fd67a811148041d47e4bf58db5185d38488aae405e607e931877c5527a78c'}, {'marker': 'topal.policy-inputs.image-library.47', 'object_section': 184, 'address': 46246320, 'bytes': 81, 'executable': True, 'object_sha256': '525fbd3ecaacbd2987dfa5ee26c69dcc5869c8a725048f603aa9dbc003eab500', 'linked_sha256': '2b3955e9f4acd403949e86f1019a62f4edc39293356c74dc4b9818abdc3667f5'}, {'marker': 'topal.policy-inputs.image-library.48', 'object_section': 188, 'address': 46246416, 'bytes': 64, 'executable': True, 'object_sha256': 'd55e8e427d281eb4fc3fc10c38a49b3e216c8ecd795c361180a9ec5bceac73ed', 'linked_sha256': 'faed22e925e1f0c3b2df787985460a07b2ce63aec880558855b26eef2691dfd1'}, {'marker': 'topal.policy-inputs.image-library.49', 'object_section': 192, 'address': 46246480, 'bytes': 52, 'executable': True, 'object_sha256': 'f2cda79ae4292f910af93b0dcf3fe562bd3b4cca0075abf69abe1b861bbe4d7c', 'linked_sha256': '015eb6b601b018319341ebd1ba105125ea066bf347d2e763b0663a95d763a273'}, {'marker': 'topal.policy-inputs.image-library.50', 'object_section': 196, 'address': 46246544, 'bytes': 81, 'executable': True, 'object_sha256': '0ed710d5b329e9ac85f36ae49b1ccbe0dad1ef3746e31073be8d914ef518ee8e', 'linked_sha256': '47b2234f70c9474b3fc4ce2fdde9f200f13ef7b40d7e52868307b5f08c2d68fa'}, {'marker': 'topal.policy-inputs.image-library.51', 'object_section': 200, 'address': 46246640, 'bytes': 81, 'executable': True, 'object_sha256': '850a90578aa497867c3b31d515207c6c0f4980c847feb46d286a6cece88068f6', 'linked_sha256': 'b327e2867067b670f3e3ef3648a6c4b309d84ee90c55f96dc68e434d7dbe7731'}, {'marker': 'topal.policy-inputs.image-library.52', 'object_section': 204, 'address': 46246736, 'bytes': 64, 'executable': True, 'object_sha256': 'd55e8e427d281eb4fc3fc10c38a49b3e216c8ecd795c361180a9ec5bceac73ed', 'linked_sha256': '3e488510496945650e2247d1dcc570ee529be5dd383755704959f7971c3f7c17'}, {'marker': 'topal.policy-inputs.image-library.53', 'object_section': 208, 'address': 46246800, 'bytes': 52, 'executable': True, 'object_sha256': 'f2cda79ae4292f910af93b0dcf3fe562bd3b4cca0075abf69abe1b861bbe4d7c', 'linked_sha256': '015eb6b601b018319341ebd1ba105125ea066bf347d2e763b0663a95d763a273'}, {'marker': 'topal.policy-inputs.image-library.54', 'object_section': 212, 'address': 46246864, 'bytes': 81, 'executable': True, 'object_sha256': 'a9764839687fa1f61fb2fefec23a11c55a39fdcc9d6b8d9a348e854c37bcca8f', 'linked_sha256': 'd27b8c1181007438fc886498e080af450e1f0900dc578c63871ffcf4d0e4ece6'}, {'marker': 'topal.policy-inputs.image-library.55', 'object_section': 216, 'address': 46246960, 'bytes': 81, 'executable': True, 'object_sha256': '2083da544d98727c6ce159da8e848e3ee9c787cf66409907bba0da31192a6812', 'linked_sha256': '77083f9f1d30f32a985340ab25890e041d5ef46d842b1b6628b7a1bd649db845'}, {'marker': 'topal.policy-inputs.image-library.56', 'object_section': 220, 'address': 46247056, 'bytes': 64, 'executable': True, 'object_sha256': 'd55e8e427d281eb4fc3fc10c38a49b3e216c8ecd795c361180a9ec5bceac73ed', 'linked_sha256': '8620bcdbc86618467c5ba44f1c2a16cb6155b80327d50d5dc24805957d114e20'}, {'marker': 'topal.policy-inputs.image-library.57', 'object_section': 224, 'address': 46247120, 'bytes': 52, 'executable': True, 'object_sha256': 'f2cda79ae4292f910af93b0dcf3fe562bd3b4cca0075abf69abe1b861bbe4d7c', 'linked_sha256': '015eb6b601b018319341ebd1ba105125ea066bf347d2e763b0663a95d763a273'}, {'marker': 'topal.policy-inputs.image-library.58', 'object_section': 228, 'address': 46247184, 'bytes': 96, 'executable': True, 'object_sha256': '120bf5a4d21291932f1d180fc36944938e7ce5d007ffce87a86baddad32e47b3', 'linked_sha256': '4c43fbeddfd1e85051b4973319bcbbaf486ea6c1604d59d722956cd6b049c15c'}, {'marker': 'topal.policy-inputs.image-library.59', 'object_section': 232, 'address': 46247280, 'bytes': 97, 'executable': True, 'object_sha256': '36c195816a3fb237dfb635d809c62dc24cfe46f5ad434a5602ee6148d527c612', 'linked_sha256': '4a1d731b336422a539cf2125aee9b4c55cd28919a5db5ffbf78409b63c0881c0'}, {'marker': 'topal.policy-inputs.image-library.60', 'object_section': 236, 'address': 46247392, 'bytes': 97, 'executable': True, 'object_sha256': '36c195816a3fb237dfb635d809c62dc24cfe46f5ad434a5602ee6148d527c612', 'linked_sha256': '8bcf377770afd0c750c1d56e4555e48f77889e382164261945ea1a15ec53bd2d'}, {'marker': 'topal.policy-inputs.image-library.61', 'object_section': 240, 'address': 46247504, 'bytes': 97, 'executable': True, 'object_sha256': '36c195816a3fb237dfb635d809c62dc24cfe46f5ad434a5602ee6148d527c612', 'linked_sha256': 'bb7651e48023e1318360cead114fc5491016d8567d4e26a458a43bd1925f9b7e'}, {'marker': 'topal.policy-inputs.image-library.62', 'object_section': 244, 'address': 46247616, 'bytes': 96, 'executable': True, 'object_sha256': '0b7f724ed33f0918019138f7f4f4efc4da3980d58e3b8cb659b392941729682e', 'linked_sha256': '000d157005081ce2e44f08122a3e1a240c3ee17cca6963e65cf8e08c0e944a98'}, {'marker': 'topal.policy-inputs.image-library.63', 'object_section': 248, 'address': 46247712, 'bytes': 97, 'executable': True, 'object_sha256': '36c195816a3fb237dfb635d809c62dc24cfe46f5ad434a5602ee6148d527c612', 'linked_sha256': '4d575a47a6cb1bc2d4564f864698f847f1f9558700fd4f8a3bfefb00d9bf6cfd'}, {'marker': 'topal.policy-inputs.image-library.64', 'object_section': 252, 'address': 46247824, 'bytes': 97, 'executable': True, 'object_sha256': '36c195816a3fb237dfb635d809c62dc24cfe46f5ad434a5602ee6148d527c612', 'linked_sha256': 'ad810e6c2044c5d388aa5a22bb746121e7a10bb46d0b0f2885b1d67f16f228c4'}, {'marker': 'topal.policy-inputs.image-library.65', 'object_section': 256, 'address': 46247936, 'bytes': 97, 'executable': True, 'object_sha256': '36c195816a3fb237dfb635d809c62dc24cfe46f5ad434a5602ee6148d527c612', 'linked_sha256': '3765a69ca486bc6ca4f53c2037433726663afd5b85ba7a5e748e0c038666916d'}, {'marker': 'topal.policy-inputs.image-library.66', 'object_section': 260, 'address': 46248048, 'bytes': 31, 'executable': True, 'object_sha256': '4d14ab4dbcfca365f201c9a9c9d3368a4042ba9c62f25c5e06a4723cb59808ec', 'linked_sha256': 'fa10e55d45896c66ce96e35101ffaf601cdec5a57519da6adb57425f3b61d995'}, {'marker': 'topal.policy-inputs.image-library.67', 'object_section': 264, 'address': 46248080, 'bytes': 289, 'executable': True, 'object_sha256': '566af9921a6f0763f90ff0ce28d8f9d8472e2fa8397e63ed72b6b942f19bca81', 'linked_sha256': '37dc41aa6403c35473b5b8bb2668d9067393b8d480c2642f3db384d03713f2e2'}, {'marker': 'topal.policy-inputs.image-library.68', 'object_section': 268, 'address': 44994272, 'bytes': 528, 'executable': False, 'object_sha256': '288fbf215d51bfb048d80b029b81a395385a012b433251dcb0bcab905ef03744', 'linked_sha256': '288fbf215d51bfb048d80b029b81a395385a012b433251dcb0bcab905ef03744'}, {'marker': 'topal.policy-inputs.image-library.69', 'object_section': 269, 'address': 44994800, 'bytes': 336, 'executable': False, 'object_sha256': '52a3e0804d93dc525ec3c67ef8ac5b01756ecf0513e36f3c19435e4c82cb5d29', 'linked_sha256': '52a3e0804d93dc525ec3c67ef8ac5b01756ecf0513e36f3c19435e4c82cb5d29'}]}, {'gs_slot': 984, 'argument': 17, 'entry_stack_offset': 88, 'native_export': 'topal_kernel_static_linux_dispatch_policy_v6', 'target': 46266224, 'sections': [{'marker': 'topal.policy-inputs.dispatch-library.0', 'object_section': 3, 'address': 46253248, 'bytes': 73, 'executable': True, 'object_sha256': 'cb9c496ef5234e5129bb5b82975701cdc84ec32909a090bae9dd90fe1a7490e4', 'linked_sha256': 'cb9c496ef5234e5129bb5b82975701cdc84ec32909a090bae9dd90fe1a7490e4'}, {'marker': 'topal.policy-inputs.dispatch-library.1', 'object_section': 6, 'address': 46253328, 'bytes': 79, 'executable': True, 'object_sha256': '872017bf481b6c1c97279bcc4acfe82207572829807084c4c2f4dbb1c363ac44', 'linked_sha256': 'eade6cc6266b90b192431ca7c7c132d9e58ea8d0cec74d2068217f9da9c80b38'}, {'marker': 'topal.policy-inputs.dispatch-library.2', 'object_section': 10, 'address': 46253408, 'bytes': 127, 'executable': True, 'object_sha256': '767de80f9cb4df09613ff8c7490087362ebee98e020ec3fb650a2d8943340549', 'linked_sha256': '4777fe0d481dfe9e42741cdfbaa675952e09fd48a76b562ef7dc1e712c8a3909'}, {'marker': 'topal.policy-inputs.dispatch-library.3', 'object_section': 14, 'address': 46253536, 'bytes': 158, 'executable': True, 'object_sha256': 'f1682ed576c8f19f011cfdca0f8ddf298a7d002fb9a27c4679050fde11d5bc6b', 'linked_sha256': 'f1682ed576c8f19f011cfdca0f8ddf298a7d002fb9a27c4679050fde11d5bc6b'}, {'marker': 'topal.policy-inputs.dispatch-library.4', 'object_section': 17, 'address': 46253696, 'bytes': 212, 'executable': True, 'object_sha256': '569a1b2edad0d2efc6b1be5b07b7f3a5384cf2b6ecc0ee81aeac92171222fe3f', 'linked_sha256': 'd5180bd7440331a2303f0d4ac673b9517ee66fba8cb7e02d59f4bab0b6f5ef86'}, {'marker': 'topal.policy-inputs.dispatch-library.5', 'object_section': 21, 'address': 46253920, 'bytes': 11, 'executable': True, 'object_sha256': 'd6cce27151649426900104a94856db51bea1ccc7fe263bfc350f8ed8268d885a', 'linked_sha256': '1ff0557e3c35e7ff1e1f3570038eda0e4c364355ddb8ae2e6245e83cfcec139b'}, {'marker': 'topal.policy-inputs.dispatch-library.6', 'object_section': 25, 'address': 46253936, 'bytes': 9, 'executable': True, 'object_sha256': '7fc921d45c8e0dac7d2aa42f26536c8830cd35bfc8bce127eab50a452af67b52', 'linked_sha256': '62fe46bbe646ed78f21f84a736529657dcdf103c3c6e9177f719571a3126c4ed'}, {'marker': 'topal.policy-inputs.dispatch-library.7', 'object_section': 29, 'address': 46253952, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '603cd969d78c949125b5e98fb74dfbb3059c64a96f7b9f5d47d914804599caf9'}, {'marker': 'topal.policy-inputs.dispatch-library.8', 'object_section': 33, 'address': 46253984, 'bytes': 62, 'executable': True, 'object_sha256': '78c942c051bb4de25f025d2aa79a84e5a9e753608200f67dce79ed294716d61b', 'linked_sha256': '3af36b5086fa0076169cff44afb53ea3eca527aca0a299ffb6a1ef1386a1965d'}, {'marker': 'topal.policy-inputs.dispatch-library.9', 'object_section': 37, 'address': 46254048, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '02adb75e6aece660e1ea384a07d5e066f48ff9460ada85dd7d91bee2788e3fb4'}, {'marker': 'topal.policy-inputs.dispatch-library.10', 'object_section': 41, 'address': 46254080, 'bytes': 62, 'executable': True, 'object_sha256': '8874002e10f349c2dcd942df314280352043d9acc97405cd2334b3d314eebaee', 'linked_sha256': '8d1135faa2fc199790ef2393c5ec97966ca46d77c33498c953bb51f75273d78e'}, {'marker': 'topal.policy-inputs.dispatch-library.11', 'object_section': 45, 'address': 46254144, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'b49db7fb72370d6d94c3982df0966765bee32337796085c8acc859b8af38441a'}, {'marker': 'topal.policy-inputs.dispatch-library.12', 'object_section': 49, 'address': 46254224, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': 'a02b7269b52d504504a600f911aaa388293a976e50b7ec936ca50fdd11ef8170'}, {'marker': 'topal.policy-inputs.dispatch-library.13', 'object_section': 53, 'address': 46254256, 'bytes': 62, 'executable': True, 'object_sha256': 'c863cf84a6240d504decd260695f69822323618ef74e33fa20392b0116b8fd52', 'linked_sha256': 'a6d85a83cfce226f340b9d08034ff56c7600b19434bf12cb86b74b3f2bb75d7e'}, {'marker': 'topal.policy-inputs.dispatch-library.14', 'object_section': 57, 'address': 46254320, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '76e92bf48f8cda4e6f8b0ff18fe798c1eff59c117ea604a4dedd3c7e5f3ba4b6'}, {'marker': 'topal.policy-inputs.dispatch-library.15', 'object_section': 61, 'address': 46254352, 'bytes': 62, 'executable': True, 'object_sha256': '09373aa1b668368890c6aa93c489e4507554c23b4515a9e7541de01068cd9df5', 'linked_sha256': 'fc8b87e970169537815d589a677a47e8a79e33ddd2c52d991665fdaf74dc279c'}, {'marker': 'topal.policy-inputs.dispatch-library.16', 'object_section': 65, 'address': 46254416, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '18fa49a658a51b1bb1e0ded56e1d40f87d00907d23df457e5869ad06a213c538'}, {'marker': 'topal.policy-inputs.dispatch-library.17', 'object_section': 69, 'address': 46254496, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'f333144cf234d381f7f59f4d6e99df09848be18a4863e1dcba39c4313a15dbec'}, {'marker': 'topal.policy-inputs.dispatch-library.18', 'object_section': 73, 'address': 46254576, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '52089e6798e95336bf318a53de2cbc77049ea564178f7c53cada4076b178d985'}, {'marker': 'topal.policy-inputs.dispatch-library.19', 'object_section': 77, 'address': 46254608, 'bytes': 62, 'executable': True, 'object_sha256': '8ccf63e61842b223016890eeea0ed70cd9892c2c540fffbf962df4d184431a36', 'linked_sha256': 'e8feb1504971f5c8bc8172bba25ecea2a216339ef17c6ee281b6860d79c28040'}, {'marker': 'topal.policy-inputs.dispatch-library.20', 'object_section': 81, 'address': 46254672, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': 'a3d5207c8c2c64b572b43ad3c605393a90b4e71afe648166b06d2ee131a63652'}, {'marker': 'topal.policy-inputs.dispatch-library.21', 'object_section': 85, 'address': 46254704, 'bytes': 62, 'executable': True, 'object_sha256': 'df4b550592662e3d9e5b0468c40b31e8dd4cce52b367f18500d4bc4efcfbbd0f', 'linked_sha256': '26514a874bb7c79c7dae0e118194730c212d5572c604a19f14ed02e01415616b'}, {'marker': 'topal.policy-inputs.dispatch-library.22', 'object_section': 89, 'address': 46254768, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '2ead3dfd296301c84b0c5e97b671ef3b312d5d516d1308a5c27ff250d1f6b1ef'}, {'marker': 'topal.policy-inputs.dispatch-library.23', 'object_section': 93, 'address': 46254848, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': 'b88b9ff4cd87c01ac1a8b238f8c0df6ee51ee9a3b5495fd1add707cc2d71e3cd'}, {'marker': 'topal.policy-inputs.dispatch-library.24', 'object_section': 97, 'address': 46254880, 'bytes': 62, 'executable': True, 'object_sha256': 'b25cc07d8c48c742bf1e67c2fae49680a7d657a9004b280c9ad62ed0a5a5d232', 'linked_sha256': 'f00b7bc11ecd2dd5ef990ca694aa797c155ac6d408c5ab4145ea81be79b3a06a'}, {'marker': 'topal.policy-inputs.dispatch-library.25', 'object_section': 101, 'address': 46254944, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '2e07f30dfeaa495f714ae53117bbc63c77e57ac6a5cfacfd8fa57ba03a8aaa80'}, {'marker': 'topal.policy-inputs.dispatch-library.26', 'object_section': 105, 'address': 46254976, 'bytes': 62, 'executable': True, 'object_sha256': '6237b02f4496ffd123e8df21d4ae7fc1896b43a0bcea473e8a114c5dce77f6d0', 'linked_sha256': '770a6c355062ba0271b073f1f9387ea7606b90c6386ad1d7b1a0c0a94a1c9efa'}, {'marker': 'topal.policy-inputs.dispatch-library.27', 'object_section': 109, 'address': 46255040, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '4ba620f0e7175c1f220d85a03c1d4f94837979bc9766756e520038e86432a464'}, {'marker': 'topal.policy-inputs.dispatch-library.28', 'object_section': 113, 'address': 46255120, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '4cc9e0f7622078b623534e92a07c5c1cd24d4d5df3f1a209649540455d617539'}, {'marker': 'topal.policy-inputs.dispatch-library.29', 'object_section': 117, 'address': 46255200, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'e3be4b97989830660e26f997868de1dc83c0636a7912276440ef497b78d43c7b'}, {'marker': 'topal.policy-inputs.dispatch-library.30', 'object_section': 121, 'address': 46255280, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': 'c07c912d18de35ef616fcfdac4a3546d6ea485213c49486d42de7aeeda2d6637'}, {'marker': 'topal.policy-inputs.dispatch-library.31', 'object_section': 125, 'address': 46255312, 'bytes': 62, 'executable': True, 'object_sha256': 'bd8cab91bec89c64780e6530892a834094b7b575c73ba9376e5469013f16a15d', 'linked_sha256': '1d86fdd064906260ab13760c438a5389c0aec1b731f49001010061df8d8730c8'}, {'marker': 'topal.policy-inputs.dispatch-library.32', 'object_section': 129, 'address': 46255376, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '2be7bedf299eb325ed059128653231511b744122b0bf5c032dee91ee000cacc3'}, {'marker': 'topal.policy-inputs.dispatch-library.33', 'object_section': 133, 'address': 46255408, 'bytes': 62, 'executable': True, 'object_sha256': '9fa80d2bb0335d943ccfb07b100a5ac19e152a813aef85e2757f1b6635fb3d30', 'linked_sha256': 'bc11f6cebbff08b78b2fefef500f5ade3cd6959164950a8d591c629d18a36b22'}, {'marker': 'topal.policy-inputs.dispatch-library.34', 'object_section': 137, 'address': 46255472, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'ae753d49f98b554b9bc16b213aaf4a2216b570481f550b99fa1f9eb6d90297bf'}, {'marker': 'topal.policy-inputs.dispatch-library.35', 'object_section': 141, 'address': 46255552, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '20c96fb8a22846b2b220585f792dc003935cd1853fab5ac9aea6aadb641aaa10'}, {'marker': 'topal.policy-inputs.dispatch-library.36', 'object_section': 145, 'address': 46255584, 'bytes': 62, 'executable': True, 'object_sha256': 'a8d7b517e3ddc5762cd30ea333484b56876d284820909f08acc9ae52d655b35c', 'linked_sha256': '191376cf7d747f376b6ea22b6b78207d9bb8516f7f8a9972848e6b5ceb999aa7'}, {'marker': 'topal.policy-inputs.dispatch-library.37', 'object_section': 149, 'address': 46255648, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': 'b253cdb93354c1c682485eb162166907752998eddcb03a179e83b03aa2ece06c'}, {'marker': 'topal.policy-inputs.dispatch-library.38', 'object_section': 153, 'address': 46255680, 'bytes': 62, 'executable': True, 'object_sha256': 'bcf2baf6a30ce5fa3b8e928b36e293b60f63ac90e4a07935b2e38b9fd5f5ad20', 'linked_sha256': '5456ca868ccd424fb791a5b44beeb3b408c9274065b2ac7a57720cf1a46d3a18'}, {'marker': 'topal.policy-inputs.dispatch-library.39', 'object_section': 157, 'address': 46255744, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '59b734497c6ef1998738e899f09ff60c9f11edc14f7fc827c46fa502d6e98995'}, {'marker': 'topal.policy-inputs.dispatch-library.40', 'object_section': 161, 'address': 46255824, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '0093062ab5dedb6340f84544458cacca96230c2daf00858bb2874975b3ee0a23'}, {'marker': 'topal.policy-inputs.dispatch-library.41', 'object_section': 165, 'address': 46255904, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '7187e825f78eee560bcc48d19b07e7e5d05dcb6f7816341f23bf6dda313b8da4'}, {'marker': 'topal.policy-inputs.dispatch-library.42', 'object_section': 169, 'address': 46255936, 'bytes': 62, 'executable': True, 'object_sha256': '14895fc5d8557f78d7cea134cb950587bd8e84dfa9af3ee90dde27a998cd8417', 'linked_sha256': '207ebdbc359380fca667cb23101973c6fdfd6504ab4ed089a314c4c5a402a426'}, {'marker': 'topal.policy-inputs.dispatch-library.43', 'object_section': 173, 'address': 46256000, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': 'bf29e1fbe32c75ef00f40501b2f485f08f10e09e3006864c75beb7fc90f06a9e'}, {'marker': 'topal.policy-inputs.dispatch-library.44', 'object_section': 177, 'address': 46256032, 'bytes': 62, 'executable': True, 'object_sha256': '391401fb508b74483a26f7f21e9554e303d6f2958fed44ca4cfd15f97e29da6c', 'linked_sha256': 'd7f201d7aa0e126b26afc7aa7c2d3884ad612116bde7f15dc37e77652281b1db'}, {'marker': 'topal.policy-inputs.dispatch-library.45', 'object_section': 181, 'address': 46256096, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'f4d9fdd21ac389b6a23583969451ce2a8f08eec452ba87df1db9d85f909f5a80'}, {'marker': 'topal.policy-inputs.dispatch-library.46', 'object_section': 185, 'address': 46256176, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '0dae54d1d35e09a77d85282c5cb804a2939b6e270c7d08a87ecbf5c02e9119f8'}, {'marker': 'topal.policy-inputs.dispatch-library.47', 'object_section': 189, 'address': 46256208, 'bytes': 62, 'executable': True, 'object_sha256': 'e1704b3773f853b53952ab2da83d3120d4736c1b5a5abbb8ef5bb7517768d1b5', 'linked_sha256': 'eb12e5f8d99ac27237753e90a8287cdc49e07757ac1f9796842007a6448d05df'}, {'marker': 'topal.policy-inputs.dispatch-library.48', 'object_section': 193, 'address': 46256272, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': 'e25223410bce4814b136a4a6498389cd2fdf34d092902be90673e1abc687b14e'}, {'marker': 'topal.policy-inputs.dispatch-library.49', 'object_section': 197, 'address': 46256304, 'bytes': 62, 'executable': True, 'object_sha256': 'd2a993620bff0c99ea02d3d61d9cb5bceb6a263e9b3e22be35eb722c9da9934e', 'linked_sha256': '7c90128416c6c857514b7210aa200597882603b4a6643d129cfa49ef18db4fcc'}, {'marker': 'topal.policy-inputs.dispatch-library.50', 'object_section': 201, 'address': 46256368, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '09ac5749e56bb485116f32241bfc54e268734005cc7213c34bf016b4a7e6ccc9'}, {'marker': 'topal.policy-inputs.dispatch-library.51', 'object_section': 205, 'address': 46256400, 'bytes': 62, 'executable': True, 'object_sha256': 'd560e239f8dfe4931bfd14d7e335748bfeb92310e8f5ec4a6c30589104d8bd06', 'linked_sha256': '17bc4a88c161754c21eebf3a2d92477c364a3c6627c4f5b0e73afb75a22a414c'}, {'marker': 'topal.policy-inputs.dispatch-library.52', 'object_section': 209, 'address': 46256464, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'a80560d2ed5304f95c554dae685f57a90367d2ac170bd8d2ba01f74c930e3e97'}, {'marker': 'topal.policy-inputs.dispatch-library.53', 'object_section': 213, 'address': 46256544, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'fd665494418365f77551eb6cb02035433c6c7cbbe46d88b8a3337e76d2254182'}, {'marker': 'topal.policy-inputs.dispatch-library.54', 'object_section': 217, 'address': 46256624, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'c3770addd20293da7704d19a33f20764abd2fe6c6758372055d91b6bd2233ea5'}, {'marker': 'topal.policy-inputs.dispatch-library.55', 'object_section': 221, 'address': 46256704, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'e969b4e703ef6c1469973b8eaa7c32f4f43f818926be25d317b94af53ea6bd21'}, {'marker': 'topal.policy-inputs.dispatch-library.56', 'object_section': 225, 'address': 46256784, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'a7cf4ae846993460b4be8e4898f524ec9bd4e6a9764f8127373716a2cdcde90f'}, {'marker': 'topal.policy-inputs.dispatch-library.57', 'object_section': 229, 'address': 46256864, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': 'bf6522df94594187200aa4c9ec6f4a20abdc30fda8dddc87f67c04b570fc261b'}, {'marker': 'topal.policy-inputs.dispatch-library.58', 'object_section': 233, 'address': 46256896, 'bytes': 62, 'executable': True, 'object_sha256': 'fe549cfc334bce591d3faa46895475ede4ce49fe77507791b316256d7b029a78', 'linked_sha256': '5f1f73b3ce702d7c539f5ecd6e03cc8b933e45fcdc81483420a2b350daa90fd7'}, {'marker': 'topal.policy-inputs.dispatch-library.59', 'object_section': 237, 'address': 46256960, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '1027c6c37170ff65958ad159907fac8c2848c14ac0322f34636c7b68d538b240'}, {'marker': 'topal.policy-inputs.dispatch-library.60', 'object_section': 241, 'address': 46256992, 'bytes': 62, 'executable': True, 'object_sha256': 'db29efeedad9d3c5dad7a4d4da09a4f1e2226530fd325ff28db5c1be0533970d', 'linked_sha256': '567fe19af41733f1546eaebdf7b0c826dd23a1cb87d3cf4a2c6268619a8e0574'}, {'marker': 'topal.policy-inputs.dispatch-library.61', 'object_section': 245, 'address': 46257056, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '0fa1c805df3ca929bb826e303f6a5bdd4125e1fcce689874d7303670bce83402'}, {'marker': 'topal.policy-inputs.dispatch-library.62', 'object_section': 249, 'address': 46257136, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '1b4863330d4c07f0beb306057a2794b429c42eda87c367a24f5fdc8d0bcc0660'}, {'marker': 'topal.policy-inputs.dispatch-library.63', 'object_section': 253, 'address': 46257168, 'bytes': 62, 'executable': True, 'object_sha256': 'afb03ec1ca95727bdb50541f29ad4061ebd81404bc5cbaff1625f92fd34cf219', 'linked_sha256': '035c7a86f84cc6494be49a5833604c3b9ee503109e83e6587139aea0392be8e2'}, {'marker': 'topal.policy-inputs.dispatch-library.64', 'object_section': 257, 'address': 46257232, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '3558a5b07f8b713e308ee60c1862b93bf5fbf390dfcebcf6c494bad05ea0df45'}, {'marker': 'topal.policy-inputs.dispatch-library.65', 'object_section': 261, 'address': 46257264, 'bytes': 62, 'executable': True, 'object_sha256': '2edb1b4a1cb2111dfcbccc7423172db2e9d2f8f6f6db495257b5027a5feec46e', 'linked_sha256': 'b4d68a1a5318617d2baba12a2e76c138c56400567a7f4e7db102a06d54ad518d'}, {'marker': 'topal.policy-inputs.dispatch-library.66', 'object_section': 265, 'address': 46257328, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '06588fd0a137117fb0f9f1b47ce64034636e8fb96b4313cd7af0060bfdee81ef'}, {'marker': 'topal.policy-inputs.dispatch-library.67', 'object_section': 269, 'address': 46257408, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'ddf6b172f04457b339606345b2da49f7b80396c7b28376ce679c0f0c5a029420'}, {'marker': 'topal.policy-inputs.dispatch-library.68', 'object_section': 273, 'address': 46257488, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': 'f3f391e14cd3078bb6c459b0a53ac0c8c2b70e8f5aa94c5de54d34f5056949cc'}, {'marker': 'topal.policy-inputs.dispatch-library.69', 'object_section': 277, 'address': 46257520, 'bytes': 62, 'executable': True, 'object_sha256': 'c6f8fe646d8f7426cab00b22878c1413181d95e52995972530954f145d2be2e3', 'linked_sha256': 'e232cbf4df02ac61c125cc37fd3c8636f1a2a8d5264095b046671f5bf5e749c4'}, {'marker': 'topal.policy-inputs.dispatch-library.70', 'object_section': 281, 'address': 46257584, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': 'f474c7b32690cf87ea771f66abc1c2a46bb0ac27787072721c74fce1fc86b483'}, {'marker': 'topal.policy-inputs.dispatch-library.71', 'object_section': 285, 'address': 46257616, 'bytes': 62, 'executable': True, 'object_sha256': '536c6703b5c22aeb87d22be939ebc73c7c549ebc55daaa13fee589f59507b128', 'linked_sha256': 'dfd9e7451868f5229ba2c177bfce11978e076061d1b4ec64957e6e08a037a415'}, {'marker': 'topal.policy-inputs.dispatch-library.72', 'object_section': 289, 'address': 46257680, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '33581423f4025efdba2ce3ee04cf086921ec861aaa24727bfc9bdd73a3bc4607'}, {'marker': 'topal.policy-inputs.dispatch-library.73', 'object_section': 293, 'address': 46257760, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '1ab3768b56723d061f691bb7dae40571f37207cf09d2222fca7a94beebc830da'}, {'marker': 'topal.policy-inputs.dispatch-library.74', 'object_section': 297, 'address': 46257792, 'bytes': 62, 'executable': True, 'object_sha256': '9b4e93cc73b18e77eb1495e453f7bec0fa734324945ba60a5348a6b618e20c38', 'linked_sha256': '90e1e49b348d842ac366b9380432322c085f84502b27c66bfc305e37c23a820b'}, {'marker': 'topal.policy-inputs.dispatch-library.75', 'object_section': 301, 'address': 46257856, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': 'b2efe7b3615162a1d458fb95aec99077e33ea777ace928ce1f9292140adbbb89'}, {'marker': 'topal.policy-inputs.dispatch-library.76', 'object_section': 305, 'address': 46257888, 'bytes': 62, 'executable': True, 'object_sha256': '7e42e1d94fb7a12ca86ff21f304ca6173c9b8965f34fe121f84dd8c9a1dbcd4a', 'linked_sha256': 'f8949170675d47f781f50f607b092c06fd9081ef66a74ec8c07521d6cb9572aa'}, {'marker': 'topal.policy-inputs.dispatch-library.77', 'object_section': 309, 'address': 46257952, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': 'c3c9c762a2e1012c8910480809cc0fbfc5895ca35f6e47593aa0b80e186f98f9'}, {'marker': 'topal.policy-inputs.dispatch-library.78', 'object_section': 313, 'address': 46257984, 'bytes': 62, 'executable': True, 'object_sha256': '9c9b2f0eaca4344439ac7394fc85aac44edf39997cd58314ece4bcb36c5a523d', 'linked_sha256': '4365da15aa07ed70451191419fc873b0a3bec678e26163b6d2bed92cb204e973'}, {'marker': 'topal.policy-inputs.dispatch-library.79', 'object_section': 317, 'address': 46258048, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '50cad38c9dd5bf8983a0df1fc9cbdb28304f8249724ad10b2dc45e7771532aa4'}, {'marker': 'topal.policy-inputs.dispatch-library.80', 'object_section': 321, 'address': 46258128, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'c7f0d93d586302714b4fa5cafb12684dec894ecdbf3af1739bfc2acfb4a2304a'}, {'marker': 'topal.policy-inputs.dispatch-library.81', 'object_section': 325, 'address': 46258208, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '8c936125cdb597070474d5d8a348661bfa5b2965be6a9089f156193ebf94d60c'}, {'marker': 'topal.policy-inputs.dispatch-library.82', 'object_section': 329, 'address': 46258288, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '27dc7f46a15d5fee22405ed8c1bd637a9c8e92813c06bdd75ca97a11fa0c91fa'}, {'marker': 'topal.policy-inputs.dispatch-library.83', 'object_section': 333, 'address': 46258368, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': 'd43672b45a8202ef67b9c10da20c4821ce071f3ae229814c12f02dc1df11ec33'}, {'marker': 'topal.policy-inputs.dispatch-library.84', 'object_section': 337, 'address': 46258400, 'bytes': 62, 'executable': True, 'object_sha256': 'ba9162e079850c5f157fcf429b81cb553d3d87e0b57672dd56633aebefc37299', 'linked_sha256': '44584449b2f04764a931c95091d52a9ac39658944a92ea7e6219b3f5869258e7'}, {'marker': 'topal.policy-inputs.dispatch-library.85', 'object_section': 341, 'address': 46258464, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '31c2a2c03ee966979cbee4543a848e256358a31136214d0b6719dcfeb6f297b1'}, {'marker': 'topal.policy-inputs.dispatch-library.86', 'object_section': 345, 'address': 46258496, 'bytes': 62, 'executable': True, 'object_sha256': 'f6cf350432014d04f83700e3172197474589f8e0e491427a796a9ec2e83f62da', 'linked_sha256': '2ea41d151ac0e37bf423405c7327726ccef6bb15e4fcd00e9e12ed806e368482'}, {'marker': 'topal.policy-inputs.dispatch-library.87', 'object_section': 349, 'address': 46258560, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'a61fa9172b700646ff03cf355d6e5e2bb904fe3a6abd50d83159f8b5086a93a6'}, {'marker': 'topal.policy-inputs.dispatch-library.88', 'object_section': 353, 'address': 46258640, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': 'a0424bccfd3d895cfcaa02e55cca6220c3103d04e46ed692a660a4a0bfd761a1'}, {'marker': 'topal.policy-inputs.dispatch-library.89', 'object_section': 357, 'address': 46258672, 'bytes': 62, 'executable': True, 'object_sha256': '3c59b471a32a1be745b7f3bf59bd09dc534381608e5d3f9e1a9c691aedbf1e06', 'linked_sha256': 'dfa0fddc0641a85ce3674166d27b83835781f2ffa1406edb6d46a51c7a3bd51f'}, {'marker': 'topal.policy-inputs.dispatch-library.90', 'object_section': 361, 'address': 46258736, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '9ff0f8896ca161b9f90d4c07774b6e78fa4ab1ce03fc8b1da1e0808aafce2510'}, {'marker': 'topal.policy-inputs.dispatch-library.91', 'object_section': 365, 'address': 46258768, 'bytes': 59, 'executable': True, 'object_sha256': '997e2f23353e1198ca51144bc2909c5413a596c7b3802820b69d5048245f6ef4', 'linked_sha256': '84ccaa5588ef319e69e8b9fe11a6822a944ce6767a97a8c8dc1f7d502eb17760'}, {'marker': 'topal.policy-inputs.dispatch-library.92', 'object_section': 369, 'address': 46258832, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'd761afcb4bcdd386c39edb6044767c8a3ae57babbf1739625e0cfc91d166c043'}, {'marker': 'topal.policy-inputs.dispatch-library.93', 'object_section': 373, 'address': 46258912, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '9de4d0711b3b665ddca8600c71af95eff8667a992e39af14e76a4c0cfb26bf47'}, {'marker': 'topal.policy-inputs.dispatch-library.94', 'object_section': 377, 'address': 46258992, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': 'efed88b18b3c9ccac4c765780d2ce1979cc95bdb25cbdd78621f7a2236fd0b36'}, {'marker': 'topal.policy-inputs.dispatch-library.95', 'object_section': 381, 'address': 46259024, 'bytes': 62, 'executable': True, 'object_sha256': '4e1ae8cbb7fc853cdc0436bc9b2c6992580ada356ba8c97b3f54361c9f18e5dc', 'linked_sha256': '1e9b926fb985e1603da1f9390bc5b784a29e86cde8ef8512a8c108a637139d3d'}, {'marker': 'topal.policy-inputs.dispatch-library.96', 'object_section': 385, 'address': 46259088, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '2b7da0578806c1c08577c4f4abc8fc44097c12b83fc964d3c2025ea21bb7fb1e'}, {'marker': 'topal.policy-inputs.dispatch-library.97', 'object_section': 389, 'address': 46259120, 'bytes': 62, 'executable': True, 'object_sha256': '4b7b8b178ffb787821ccbd592c7eaa9662ca174db6124c9a8653813df5ef5619', 'linked_sha256': '04439cdeac7e9c3b8c4c85ac67aa62504c587bd076a02ff1e9463f2fe8ab9047'}, {'marker': 'topal.policy-inputs.dispatch-library.98', 'object_section': 393, 'address': 46259184, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '6127e1722bcb82a1b81e1b9535de96665b630e24febb560a812c15148fb40683'}, {'marker': 'topal.policy-inputs.dispatch-library.99', 'object_section': 397, 'address': 46259264, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '4a4f86bc4f5bb9f8c1c01d0cb760988d538694dd2a5e1f64ae57c05da2bd737e'}, {'marker': 'topal.policy-inputs.dispatch-library.100', 'object_section': 401, 'address': 46259296, 'bytes': 62, 'executable': True, 'object_sha256': '71f0908603eaecb23e0cabee06b4f87c5a2203f252f4678cbb33868495a2e16b', 'linked_sha256': '8b52f83946c0c7caf034bb9bd448b28c018e0b471d86a36ccff4d2589d454f53'}, {'marker': 'topal.policy-inputs.dispatch-library.101', 'object_section': 405, 'address': 46259360, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': 'e82f125112579cccbd46a3cf8e4e32537e5d43353a2afa855d435aae6d0f6697'}, {'marker': 'topal.policy-inputs.dispatch-library.102', 'object_section': 409, 'address': 46259392, 'bytes': 62, 'executable': True, 'object_sha256': '76b64b37ca2087f7e001bf25c750d487bc1185ae67b336caaf6b7d79bfd45d5a', 'linked_sha256': 'eb7dfb47c3e6efc2b1decd8d1a85ccc5b9765d6df363fd2a88c027b2bf90809c'}, {'marker': 'topal.policy-inputs.dispatch-library.103', 'object_section': 413, 'address': 46259456, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': 'c287ad3ae628031852d19b867b057a5a20e9d7e6dcb00cb02e2f75f88cef2be8'}, {'marker': 'topal.policy-inputs.dispatch-library.104', 'object_section': 417, 'address': 46259488, 'bytes': 62, 'executable': True, 'object_sha256': '4060100c3bf754090daa9c2c7b5ae91a7bec6dee90adad7c50ea96eb86517d96', 'linked_sha256': '9f436927f5c576036b186e5f91098790169384a4e84f1f007898a41350cb8342'}, {'marker': 'topal.policy-inputs.dispatch-library.105', 'object_section': 421, 'address': 46259552, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'a7b1628f77a190c6b17c944343ae828587178747788c64576ccc64392d345fa8'}, {'marker': 'topal.policy-inputs.dispatch-library.106', 'object_section': 425, 'address': 46259632, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '6802344adb92a06fe86c55ec9351193d2a7019d545f4fa5e8c3d2896f1065779'}, {'marker': 'topal.policy-inputs.dispatch-library.107', 'object_section': 429, 'address': 46259712, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '647bc8e23a17b240a492d9e7884a1eaf41178df43ab813a1398dd254010ebc12'}, {'marker': 'topal.policy-inputs.dispatch-library.108', 'object_section': 433, 'address': 46259792, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '3eeedeb46c437cbdb3b9c95f04b7ef9a512e50c95dd2abcdbf56f3cc28119a8f'}, {'marker': 'topal.policy-inputs.dispatch-library.109', 'object_section': 437, 'address': 46259872, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'c108b87de8a26d78a0aac2143e191d382600c909bcaf3329833546a108aa4a2d'}, {'marker': 'topal.policy-inputs.dispatch-library.110', 'object_section': 441, 'address': 46259952, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'f454e5e66ec31b161f079eb9207c275c09ea3846399fbfbf614ddaed19107e26'}, {'marker': 'topal.policy-inputs.dispatch-library.111', 'object_section': 445, 'address': 46260032, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '6c77808479a79b9644b48f069f332bd44bcb7496a11bd57e57c7f7757c90ac75'}, {'marker': 'topal.policy-inputs.dispatch-library.112', 'object_section': 449, 'address': 46260064, 'bytes': 62, 'executable': True, 'object_sha256': 'bc2f87b88be54f092a949ed1a7823a66a4283f86d96e39d4e5ddaaa4f81d3d45', 'linked_sha256': '5fcefb87f595b5f1ea70684c9169a88266d6cbdf0ba1904d384b836266e799ca'}, {'marker': 'topal.policy-inputs.dispatch-library.113', 'object_section': 453, 'address': 46260128, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '26b4f219a9ae0913bb2dea02ea37eb2aa2994f1801a652d3670ef2c311093d77'}, {'marker': 'topal.policy-inputs.dispatch-library.114', 'object_section': 457, 'address': 46260160, 'bytes': 62, 'executable': True, 'object_sha256': 'a0b36c6169e9bb1a69481b4119d4aa32696c14d9b6e6749d4a952e4af3ad8b9b', 'linked_sha256': 'b3a2f58f2081258e3071692e81ab9b72f92c2a8d6f83dc189ba71e75f7f14388'}, {'marker': 'topal.policy-inputs.dispatch-library.115', 'object_section': 461, 'address': 46260224, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'c9319af5a1bafe080d99edf83cfa63c69fb51b26d9b2d3b679211f9dc8bfb142'}, {'marker': 'topal.policy-inputs.dispatch-library.116', 'object_section': 465, 'address': 46260304, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '2527cddb6cb708528be29ae0660d08e933cd511c9010caee77aba2513956474f'}, {'marker': 'topal.policy-inputs.dispatch-library.117', 'object_section': 469, 'address': 46260336, 'bytes': 62, 'executable': True, 'object_sha256': '86476823058ee6afc768a5aa50d4325d55510536b9f11fa301758dbd48c4f663', 'linked_sha256': 'e809ec8702fe070487b5b74e56fce76ce817f294d7afb21e499eea36e84be17b'}, {'marker': 'topal.policy-inputs.dispatch-library.118', 'object_section': 473, 'address': 46260400, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '20c7e9c8e0041ae6602404332f14c71c12e6a6d2309e6f2a95d7d52adb86bcbe'}, {'marker': 'topal.policy-inputs.dispatch-library.119', 'object_section': 477, 'address': 46260432, 'bytes': 62, 'executable': True, 'object_sha256': 'f1fea4fe49cb85ae9f1b3577639691340935a3489a4f2dab78bab0c1a20773ea', 'linked_sha256': '5f7b4c6103704bd58c396ce439c51f93aac2784bdf47ebb18037488c9a7ad2a0'}, {'marker': 'topal.policy-inputs.dispatch-library.120', 'object_section': 481, 'address': 46260496, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'b70c0c9c19447720de1cde2c3a05ed48c284e67f2d15ba2ebd9265a7aa43a430'}, {'marker': 'topal.policy-inputs.dispatch-library.121', 'object_section': 485, 'address': 46260576, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'bd2310233d0fd5511a54634daeaa3324a1e3922b0a724f38d4b74bd2faf2a1fa'}, {'marker': 'topal.policy-inputs.dispatch-library.122', 'object_section': 489, 'address': 46260656, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '199d7bf3da82651b63efae56304443b4e9fb662b0be323550dfacaa0237ef57a'}, {'marker': 'topal.policy-inputs.dispatch-library.123', 'object_section': 493, 'address': 46260688, 'bytes': 62, 'executable': True, 'object_sha256': '087c73511d01393412e66f0a77499ad064d232e361b2f56dd6816d5b48575581', 'linked_sha256': '31de780cb4750cf5853281b8b4866631d72aec67e7f4aedc4e5e8e94071cd122'}, {'marker': 'topal.policy-inputs.dispatch-library.124', 'object_section': 497, 'address': 46260752, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '365e0605af796e6ee7276266b4485d4d7524d7275e8528f53cad0039d96d30aa'}, {'marker': 'topal.policy-inputs.dispatch-library.125', 'object_section': 501, 'address': 46260784, 'bytes': 62, 'executable': True, 'object_sha256': '028b81d886d1433ef5a27ba3685bd9eb60ce5072be5e13bd476006a24e001d8d', 'linked_sha256': '6c03345a9f9e6f8925fd1e51d8d654b3ca2d7a423f7cdfd27cd9d4e658c1d26c'}, {'marker': 'topal.policy-inputs.dispatch-library.126', 'object_section': 505, 'address': 46260848, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '0c11f1b7f44228dd1665090d7a6c5751910cdc45bcb3b5ea985afd99e97bf3b3'}, {'marker': 'topal.policy-inputs.dispatch-library.127', 'object_section': 509, 'address': 46260928, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '181b4b98cd341626a029c4ef16fa58f84d496d4aa3de2d8b44ad5a2a3ed1262e'}, {'marker': 'topal.policy-inputs.dispatch-library.128', 'object_section': 513, 'address': 46260960, 'bytes': 62, 'executable': True, 'object_sha256': '324448f234741bdceabada78ff21abf2b6c4bbbcb53354aec0a2001016317309', 'linked_sha256': '2fee631cbe341454a80a10bedd1e302c4b193316f275f6abc1888e8f8f264a1e'}, {'marker': 'topal.policy-inputs.dispatch-library.129', 'object_section': 517, 'address': 46261024, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '49720e8c31b94650a0afd282b9bf735b89647dc17a78913fdead5775150c049d'}, {'marker': 'topal.policy-inputs.dispatch-library.130', 'object_section': 521, 'address': 46261056, 'bytes': 62, 'executable': True, 'object_sha256': '2852c4d59642079c40963f1c7e4850d20ca25062b0ccbdc3b4f66c97b9e656de', 'linked_sha256': 'b9092ab5fdaaa3795e80186cae16ae8c10334abfe17a7387d51cd40652b4e6ac'}, {'marker': 'topal.policy-inputs.dispatch-library.131', 'object_section': 525, 'address': 46261120, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '891833b10f2ff01e8073cc693de2f2351de3416381b68b4b1fc112d6e2411c28'}, {'marker': 'topal.policy-inputs.dispatch-library.132', 'object_section': 529, 'address': 46261200, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '2c975c71208ba207ef9bb6900f6daabd19864673bfc1d45162028a13d37d87c2'}, {'marker': 'topal.policy-inputs.dispatch-library.133', 'object_section': 533, 'address': 46261280, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '3635432c5c72f006dea328f55a30b2c64c308ae8b2695bf42c98599f786a41bf'}, {'marker': 'topal.policy-inputs.dispatch-library.134', 'object_section': 537, 'address': 46261360, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': 'c85e1247a741f88114c5c136d28b98ccb6b4f5e02fc4bdb49d7d3e0a0cafb479'}, {'marker': 'topal.policy-inputs.dispatch-library.135', 'object_section': 541, 'address': 46261392, 'bytes': 62, 'executable': True, 'object_sha256': '8fc0a00debd6cf100ee15139a0e7de8fe619387dec185e77467632d445f3762b', 'linked_sha256': 'f35c372cff668da2da6e4f9f48f4f5bc22b211706b3f1c1d78cfe15b5f4673e9'}, {'marker': 'topal.policy-inputs.dispatch-library.136', 'object_section': 545, 'address': 46261456, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '6f684ee696b01ddb595391352a8dab7f5023f3e917eee19ffc780638a439ffe0'}, {'marker': 'topal.policy-inputs.dispatch-library.137', 'object_section': 549, 'address': 46261488, 'bytes': 62, 'executable': True, 'object_sha256': '90c0ee4ce37fdb87e65c997d092a398400b2e2837f311ec0b1452d8220626e52', 'linked_sha256': 'fc6efaecef5b21b7417e511c3a0787593d82409ca5ede1672baed02935cd4f75'}, {'marker': 'topal.policy-inputs.dispatch-library.138', 'object_section': 553, 'address': 46261552, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '4f809654bbea324c4803adab5425b73551cd2c73b28abdfd6298f68e8ba100b6'}, {'marker': 'topal.policy-inputs.dispatch-library.139', 'object_section': 557, 'address': 46261632, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '4f9e9165552bb33fb81178212a9b2b6286843705b12aa64ca2cf9558d57529ff'}, {'marker': 'topal.policy-inputs.dispatch-library.140', 'object_section': 561, 'address': 46261664, 'bytes': 62, 'executable': True, 'object_sha256': 'd20f7645ee04aa0773b4c45d46fc5928138123aa049e1c1baf43aa1ccee38673', 'linked_sha256': '179e0b4d4acc44e906385f3eb38d8c9ecd19fb022397a8374c3865ffcfc7da3e'}, {'marker': 'topal.policy-inputs.dispatch-library.141', 'object_section': 565, 'address': 46261728, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '0580330f74defe76f533ddc3deb6dbc93d69c4a9f9e753f35de4c60566c4ac02'}, {'marker': 'topal.policy-inputs.dispatch-library.142', 'object_section': 569, 'address': 46261760, 'bytes': 62, 'executable': True, 'object_sha256': 'b94830c199312af88fbdf9586f50c62654a47ebb893f8636bbf6caad2d2f14cf', 'linked_sha256': 'c7f08d21bb28833a379901ad58aa7b447ac111a299a3580613ae320f9c3751bf'}, {'marker': 'topal.policy-inputs.dispatch-library.143', 'object_section': 573, 'address': 46261824, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'cfba84c82634d3785dac0a6dd8e2a41c170dc2a31657b46c53fcaf737766f623'}, {'marker': 'topal.policy-inputs.dispatch-library.144', 'object_section': 577, 'address': 46261904, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '9aeacf717eb13af51dc2b91b32afd07857db8bf1d05aceee2abeab084fbee3e5'}, {'marker': 'topal.policy-inputs.dispatch-library.145', 'object_section': 581, 'address': 46261984, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '7c3d6c849fd6b826f193d6453a5bd97a7c4324e474d1077ec492f93a48ef67f1'}, {'marker': 'topal.policy-inputs.dispatch-library.146', 'object_section': 585, 'address': 46262016, 'bytes': 62, 'executable': True, 'object_sha256': '6ab586aaf48214bfe035b38aae7b666ca970f35da670713bd8f42c066c8468d3', 'linked_sha256': 'd9d45bc7f573d6a8adc58956d01f1a16a37b264b9e23518a3decc0b08a8bd6ae'}, {'marker': 'topal.policy-inputs.dispatch-library.147', 'object_section': 589, 'address': 46262080, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '34fb3ee418dc19e1922d8e972dc634194d8263c29a7f459c63c5ac9e1a9922ec'}, {'marker': 'topal.policy-inputs.dispatch-library.148', 'object_section': 593, 'address': 46262112, 'bytes': 62, 'executable': True, 'object_sha256': '12fae3eeca1fb5de19f508d5f2ad11f054024eec465158a13bdc2dd5f089c2c1', 'linked_sha256': 'bc1154e523693478efd1fb36decb754e92e62dd8597e5f7b5a5c226990dec08b'}, {'marker': 'topal.policy-inputs.dispatch-library.149', 'object_section': 597, 'address': 46262176, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '3f34f2a2dbf684ec6ac94621181f47651e54204494885d4b98868265b452f969'}, {'marker': 'topal.policy-inputs.dispatch-library.150', 'object_section': 601, 'address': 46262256, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '68fe41c4741c083299d7b3a68c2809cf2477279dcb19f6546d477c7fd0207375'}, {'marker': 'topal.policy-inputs.dispatch-library.151', 'object_section': 605, 'address': 46262288, 'bytes': 62, 'executable': True, 'object_sha256': '399123b574ec099cec2756a48487ddea18bebb31bb59d1a7c7377914d351bdc4', 'linked_sha256': 'd3d432658cd8849d0731521a90d149dd991199dac7ddc7a6d101fcad7264392e'}, {'marker': 'topal.policy-inputs.dispatch-library.152', 'object_section': 609, 'address': 46262352, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': 'abd2ee0773d72a458efde37a07311cc1ff610affc3618d21f2b6e764c984418a'}, {'marker': 'topal.policy-inputs.dispatch-library.153', 'object_section': 613, 'address': 46262384, 'bytes': 62, 'executable': True, 'object_sha256': '1ca7ef1081c919ccdfb466303bc12ae07d940d045452807cee279fcfc9522cdb', 'linked_sha256': 'f951559fffec078a378b3278eb10ceda5501e95e7006433183d770ec16523ea1'}, {'marker': 'topal.policy-inputs.dispatch-library.154', 'object_section': 617, 'address': 46262448, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': 'af8b187ecf72ff7298d48104b61a5b07edb55e7d8fbd0576907498c04dd2c1d5'}, {'marker': 'topal.policy-inputs.dispatch-library.155', 'object_section': 621, 'address': 46262480, 'bytes': 62, 'executable': True, 'object_sha256': '51ede1bd27dda8d87931db8839dd5ef81d9e9f2a49f7cdc14b2a341c039a05f9', 'linked_sha256': 'e8192062e03cc28f8f761ca5a02f855584e1da2b27076f4090261b3bb4eadbf9'}, {'marker': 'topal.policy-inputs.dispatch-library.156', 'object_section': 625, 'address': 46262544, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '51ee153aae1a020409e4106ddfdb121c154d2daa389a9cb07fb8c8bf3cd226df'}, {'marker': 'topal.policy-inputs.dispatch-library.157', 'object_section': 629, 'address': 46262624, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'd9ad230cff96a08818793e1e8be1e59151652e8dc5a260f7292dcdfa1d56c1bb'}, {'marker': 'topal.policy-inputs.dispatch-library.158', 'object_section': 633, 'address': 46262704, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '129674528d47211431f044b7dcd75e17c413e6fc7fc8ba6a120f63f2cb58be92'}, {'marker': 'topal.policy-inputs.dispatch-library.159', 'object_section': 637, 'address': 46262784, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '55829b39026b8eaf25372256c711c732f638c5e2adb047341ecc5f68a906cfb0'}, {'marker': 'topal.policy-inputs.dispatch-library.160', 'object_section': 641, 'address': 46262864, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'b1124de26671f1575d0e0eefdaa8a74fe6ff79eab3c8d6184f06f068a3c6f0f9'}, {'marker': 'topal.policy-inputs.dispatch-library.161', 'object_section': 645, 'address': 46262944, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': 'd14a75df8003143b2335f49ed07ebde7cb9f2991afb8e9ba14228b8185a11cc9'}, {'marker': 'topal.policy-inputs.dispatch-library.162', 'object_section': 649, 'address': 46262976, 'bytes': 62, 'executable': True, 'object_sha256': 'df162b7af736fe8419d9d8ba9b47875a1409c59354f81d8e17ac702f1ef34c39', 'linked_sha256': '1e76b07a0cbc7270292e60c15d89d0e11b6e83fce5d4957e85b6a33d3e964196'}, {'marker': 'topal.policy-inputs.dispatch-library.163', 'object_section': 653, 'address': 46263040, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': 'bfd8a6d5af9deb05838bf492ef932010458c6690d0b0b23006c8d3811fa61ab6'}, {'marker': 'topal.policy-inputs.dispatch-library.164', 'object_section': 657, 'address': 46263072, 'bytes': 62, 'executable': True, 'object_sha256': '09fd3e55d4c6cfd7c4648e8f2f3db2c9fc67154110529c871967693fbcc1234b', 'linked_sha256': '6c860d6369b55eab274440c4cc87999d9bbb3d397341ad78a3c0c59e72adb4c6'}, {'marker': 'topal.policy-inputs.dispatch-library.165', 'object_section': 661, 'address': 46263136, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'dce1d2b47ab9de9148747c1f78892e991afbee235f4727fc6a9a2bc6471291fe'}, {'marker': 'topal.policy-inputs.dispatch-library.166', 'object_section': 665, 'address': 46263216, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '2df36036d9f27a3d132871d6d853dc096911476ef875f32c022f9e4d15330dde'}, {'marker': 'topal.policy-inputs.dispatch-library.167', 'object_section': 669, 'address': 46263248, 'bytes': 62, 'executable': True, 'object_sha256': 'b3147051f86238b6ee46c10985f9dc9d77f4d6b087693e261764df5c8c76dea9', 'linked_sha256': '3081e4861778d9e856bde4082f246df2f534ed8bc3bcd3d9119f1c0d3c9016fc'}, {'marker': 'topal.policy-inputs.dispatch-library.168', 'object_section': 673, 'address': 46263312, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '183ba6a89e3edfaca99371cfa1beb5e01b101d42389453a7667e04373e9dec25'}, {'marker': 'topal.policy-inputs.dispatch-library.169', 'object_section': 677, 'address': 46263344, 'bytes': 62, 'executable': True, 'object_sha256': 'e6294bb27199776558593ffe6d7a20ac550f312c4c5070dbac1f9df0aaebadc5', 'linked_sha256': 'fec2964131e469a3f26574bf0b2c31a2b5ea3000a90ad8b46fd10c5701d0837d'}, {'marker': 'topal.policy-inputs.dispatch-library.170', 'object_section': 681, 'address': 46263408, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '7896d985e7ad47229ce48322bca47cc042fb58b4a2a4a3e59e0f0f3a113ae185'}, {'marker': 'topal.policy-inputs.dispatch-library.171', 'object_section': 685, 'address': 46263488, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'efa7b110b29c84f258358743130833254bb227218de613bd556327109c05a0be'}, {'marker': 'topal.policy-inputs.dispatch-library.172', 'object_section': 689, 'address': 46263568, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': 'bab88130c73740cbadcfe4d2ea04b772790005af45992a5781ab750ec2b217ab'}, {'marker': 'topal.policy-inputs.dispatch-library.173', 'object_section': 693, 'address': 46263600, 'bytes': 62, 'executable': True, 'object_sha256': '352d9f1e3d62f740a1b8a88f60c12fc02e39ce9aa1db2af291ac58532acad8bd', 'linked_sha256': '12ee94a4b367206246333c4c7da03bf364bb15b274d97b8cca1b65f7d6dc22ec'}, {'marker': 'topal.policy-inputs.dispatch-library.174', 'object_section': 697, 'address': 46263664, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '6e7ba7a1db7321139deafa210c9c661b497a7b283f338fd8ac7f1ff1e8e84361'}, {'marker': 'topal.policy-inputs.dispatch-library.175', 'object_section': 701, 'address': 46263696, 'bytes': 62, 'executable': True, 'object_sha256': '9a7a323011c7709faeff1dcfadd8f37f5cba4b3ea7110a715d01e4acc6c5f866', 'linked_sha256': 'b04aa575d63548c2cd70acb40d88cbed9fa90290ead7e7ea1faeee34712d700e'}, {'marker': 'topal.policy-inputs.dispatch-library.176', 'object_section': 705, 'address': 46263760, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'f06d13a316aed9ff003485e2046cf4936d88539e027cef256b6662549a9e7f46'}, {'marker': 'topal.policy-inputs.dispatch-library.177', 'object_section': 709, 'address': 46263840, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '2d01e6a5399e595e4fb712872de59d030a98ae9e771804cff2f3ff0d61301fca'}, {'marker': 'topal.policy-inputs.dispatch-library.178', 'object_section': 713, 'address': 46263872, 'bytes': 62, 'executable': True, 'object_sha256': '924396add3c770cd2cb310bd2f0a82831e34a66374aac3d55267f4e888f1f2f9', 'linked_sha256': '3a0051d8993599c03d024ea1fc365c24981450ff15a74094abcc80249bffab2b'}, {'marker': 'topal.policy-inputs.dispatch-library.179', 'object_section': 717, 'address': 46263936, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': 'b627f1172036b1d63afad240752efc87d87b41ce85e3b95e0d6756f0f9c4bcf1'}, {'marker': 'topal.policy-inputs.dispatch-library.180', 'object_section': 721, 'address': 46263968, 'bytes': 62, 'executable': True, 'object_sha256': 'b7949057a12dee5a73b1df636eff8765bd7ce7646c875171d18fca6992512fb8', 'linked_sha256': '7311053ad603d4e889ddb54133722927037f7eb25c9393ba1a6971d54d989bf3'}, {'marker': 'topal.policy-inputs.dispatch-library.181', 'object_section': 725, 'address': 46264032, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '8eb3366cd356d6e418ff05ba86fdc285c5bedf26a6117d2d06f12f754a9d94b4'}, {'marker': 'topal.policy-inputs.dispatch-library.182', 'object_section': 729, 'address': 46264064, 'bytes': 62, 'executable': True, 'object_sha256': '05933d813eca728639ab03917ed922842fe3088bcdecf2558c01a8b22458be0b', 'linked_sha256': '444ccda19fc181bb29b7a9daac1d209c2addde1b8c3cfa1250c49157592594c3'}, {'marker': 'topal.policy-inputs.dispatch-library.183', 'object_section': 733, 'address': 46264128, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'f8354d88f791ee040119f6cdff4331bebabcacb0d915fb4389df1e178e253281'}, {'marker': 'topal.policy-inputs.dispatch-library.184', 'object_section': 737, 'address': 46264208, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '48078b79ad44c5c982090ea83fe5f18ea172cbbef53d11bc2d7db7bad9e9db66'}, {'marker': 'topal.policy-inputs.dispatch-library.185', 'object_section': 741, 'address': 46264288, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'fe9b9dd8665a16b54ecf22605a675b8df0506714d7ad384822c98fd05ea3507a'}, {'marker': 'topal.policy-inputs.dispatch-library.186', 'object_section': 745, 'address': 46264368, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '71d2791826dfdf6852a0844c8dbf621b1bd8b91a3e2a9c35f4e9459d410180c8'}, {'marker': 'topal.policy-inputs.dispatch-library.187', 'object_section': 749, 'address': 46264448, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '80bbb5b50f029954b94e2519418e2165b630fc88bbab5bf16db8bd1eba3796d0'}, {'marker': 'topal.policy-inputs.dispatch-library.188', 'object_section': 753, 'address': 46264480, 'bytes': 62, 'executable': True, 'object_sha256': '0315156f0dcbbb006c60d2e38241c18d593eaa8720425da8aca56b85f4dfd23f', 'linked_sha256': 'b3a6e5001d504c143449c76c2f65e13f8443ff8c57b63eed89a3f109a2af9b48'}, {'marker': 'topal.policy-inputs.dispatch-library.189', 'object_section': 757, 'address': 46264544, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '9440f0fb1fd26ddeb6d9b69cc7812eeedbe272c584ab1558c46e6a96b67a1161'}, {'marker': 'topal.policy-inputs.dispatch-library.190', 'object_section': 761, 'address': 46264576, 'bytes': 62, 'executable': True, 'object_sha256': '2a4cd53c41246714a28a05d640f59311a6105fde9f9c370c9dbc81273d14dd68', 'linked_sha256': '8395971f5c1bc056dc94355e3779a863df377c6e556288c255aa4a44cca90aa4'}, {'marker': 'topal.policy-inputs.dispatch-library.191', 'object_section': 765, 'address': 46264640, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '474345a2e5c7096e756eb5b37ded7e3f44adee3ffd590ce4149e918b1324c3ab'}, {'marker': 'topal.policy-inputs.dispatch-library.192', 'object_section': 769, 'address': 46264720, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': 'f63c17f37aa77d1115d53b3b9908e08eda2df55329771814fd06278e3cb12603'}, {'marker': 'topal.policy-inputs.dispatch-library.193', 'object_section': 773, 'address': 46264752, 'bytes': 62, 'executable': True, 'object_sha256': '704c7ec9e0049f61ab5a4ad0037dbb4fbd38018f2540e663166363cd6cee7700', 'linked_sha256': '0f271d7b7567161dcfa1d876aed01efaa56a9b117a50f03f1d15fa958050c090'}, {'marker': 'topal.policy-inputs.dispatch-library.194', 'object_section': 777, 'address': 46264816, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': 'd7949390a9413f9249532f015d3dcabbdab643b4f0550a673479166b8c6d813c'}, {'marker': 'topal.policy-inputs.dispatch-library.195', 'object_section': 781, 'address': 46264848, 'bytes': 62, 'executable': True, 'object_sha256': 'e6ff420028b068e7b7339f2eeff2f6a14ccf4191546123e44e2651269114095f', 'linked_sha256': '96ea07078057203794583c4f6386e94e86ed78224b7f7a4989fbf49c4fc8dec1'}, {'marker': 'topal.policy-inputs.dispatch-library.196', 'object_section': 785, 'address': 46264912, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '97bac6719c029b725f1715d15a85e36722621fc5ad0797e2a9188e3bdd656279'}, {'marker': 'topal.policy-inputs.dispatch-library.197', 'object_section': 789, 'address': 46264992, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '904032b640632c01bbf39ea367354ed3fe241b4a82ea73ef9963c9f80c2583d1'}, {'marker': 'topal.policy-inputs.dispatch-library.198', 'object_section': 793, 'address': 46265072, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '552d64fe379474e7c213475411c6a76a5689cc8cf667bd42124c964fbfdf0b66'}, {'marker': 'topal.policy-inputs.dispatch-library.199', 'object_section': 797, 'address': 46265104, 'bytes': 62, 'executable': True, 'object_sha256': 'c094dc633be348170d58d60bee8086ba00526234a5409bdc34198be586e86e88', 'linked_sha256': '2d864ed2c7aaeb1f69193e88199a65ec6606bfb87d805d1e49c842b61f58737b'}, {'marker': 'topal.policy-inputs.dispatch-library.200', 'object_section': 801, 'address': 46265168, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '490380c3dde4f9e3d3ad5b2381678799564651fdec5632dd8092220ebb6e1bec'}, {'marker': 'topal.policy-inputs.dispatch-library.201', 'object_section': 805, 'address': 46265200, 'bytes': 62, 'executable': True, 'object_sha256': '1f17aee62f4a6f439bf5c544d98aef6227b3006690373657066a3dccec0b9464', 'linked_sha256': 'da9e31f7740e9c61c9f52027c9366d97ce1214fef1743323d14e41db3b3cda13'}, {'marker': 'topal.policy-inputs.dispatch-library.202', 'object_section': 809, 'address': 46265264, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '139fbb0084410d6bde935f5344332fc6904288879c1705f77a9e69fd2f74665a'}, {'marker': 'topal.policy-inputs.dispatch-library.203', 'object_section': 813, 'address': 46265344, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '38eeb209720ace6ae931bd9f1bb0c414116f245ca99c4a7ca651a0e9a4594990'}, {'marker': 'topal.policy-inputs.dispatch-library.204', 'object_section': 817, 'address': 46265376, 'bytes': 62, 'executable': True, 'object_sha256': '32119897e3437883a31a3ccf11d51c4e3d1222b8a86347653bba76c99a89b616', 'linked_sha256': 'd72d790b6e4f1aabcb8977206cfef40e53e52e32a770bd212ba688f8c6009fb0'}, {'marker': 'topal.policy-inputs.dispatch-library.205', 'object_section': 821, 'address': 46265440, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '060104c29d28e910a68d0906f244b8b65775bad9bc6eec62ab7a78d42c08f47d'}, {'marker': 'topal.policy-inputs.dispatch-library.206', 'object_section': 825, 'address': 46265472, 'bytes': 62, 'executable': True, 'object_sha256': '14a37cbbaf9bf9c4f76ed6af6e8245990acf7f530e1d8a68f8e60f3b98caf9b4', 'linked_sha256': 'db13af15eff9f403885784c83e397a90c18671adf185c704294050721290e69e'}, {'marker': 'topal.policy-inputs.dispatch-library.207', 'object_section': 829, 'address': 46265536, 'bytes': 32, 'executable': True, 'object_sha256': 'c702a6ce35bdef05a999037553031ba5f1eb6fc55a03144e9f00a54153655124', 'linked_sha256': '6953c70bbfb714c58e2640049d85188117982301d9d402143d8a3cfd794e5a41'}, {'marker': 'topal.policy-inputs.dispatch-library.208', 'object_section': 833, 'address': 46265568, 'bytes': 62, 'executable': True, 'object_sha256': '0382049796267492cbde4308699d2a45f12142c008ed06af0e21b4df50f50994', 'linked_sha256': '59a0aeeed57ed860b20a4f8522bcfac4abe606ed9a3787098af477880ea51fbd'}, {'marker': 'topal.policy-inputs.dispatch-library.209', 'object_section': 837, 'address': 46265632, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '7a368e1fe34fdfb2dab8918e2c87b41856abaaa68327be650dc18d7c5be12348'}, {'marker': 'topal.policy-inputs.dispatch-library.210', 'object_section': 841, 'address': 46265712, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '02bbb399cfd379df15b21118bc570838d9463e699981f36bebd883f3141ccb10'}, {'marker': 'topal.policy-inputs.dispatch-library.211', 'object_section': 845, 'address': 46265792, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'c9d0ecd513e318da47acb36e6b18df6d43a9d01d0288df3e2bf7e8e7bb1e66ae'}, {'marker': 'topal.policy-inputs.dispatch-library.212', 'object_section': 849, 'address': 46265872, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '9c51ad67c6d0b5ca9d732c027ce557db5417490b6d10774412d68cd00f4195cb'}, {'marker': 'topal.policy-inputs.dispatch-library.213', 'object_section': 853, 'address': 46265952, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'd25aff6ad27ae6e4ef1bbabd3f9a9868f81ecc834acdc3a909117915eaf5fdae'}, {'marker': 'topal.policy-inputs.dispatch-library.214', 'object_section': 857, 'address': 46266032, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': 'cbb760cdc59277e7a39c4827ec7f187f08834a849314cdd167535629fa128821'}, {'marker': 'topal.policy-inputs.dispatch-library.215', 'object_section': 861, 'address': 46266112, 'bytes': 71, 'executable': True, 'object_sha256': 'c3b612d2805837ab678760cf43c20bb4868ea4fcc548f4fb00be04351f51adb2', 'linked_sha256': '37cfd753706fc8effda98b3b4916a685923055ba1be12dd9a24f6f9efcb526ee'}, {'marker': 'topal.policy-inputs.dispatch-library.216', 'object_section': 865, 'address': 46266192, 'bytes': 23, 'executable': True, 'object_sha256': 'eba42c5916581a794c2659c7ce7473392600bf19711b0785c3df3966d21c7f60', 'linked_sha256': '362b327b70832596d19a11c3d34cd5abd7fbd37021c8f69ff82f390b52bc879a'}, {'marker': 'topal.policy-inputs.dispatch-library.217', 'object_section': 869, 'address': 46266224, 'bytes': 835, 'executable': True, 'object_sha256': 'fd831869dd2add0313cc2f88968113a5272fe1b18fde3cb5db507615f07b5acb', 'linked_sha256': '5a97171f1311c4d83badae3a372ac706491d3ea747d3f3c6e4e674cb7420b748'}, {'marker': 'topal.policy-inputs.dispatch-library.218', 'object_section': 871, 'address': 44995312, 'bytes': 568, 'executable': False, 'object_sha256': '5203ba4e767d42f262e42eaca774fe88c052b116e47febe72e9dd6183d4bd22f', 'linked_sha256': '9d4d31c91b7f0ade96b518628dbd9c8a7e225fd49d862215baaf2077d1d0ae02'}, {'marker': 'topal.policy-inputs.dispatch-library.219', 'object_section': 875, 'address': 44995880, 'bytes': 3328, 'executable': False, 'object_sha256': 'a0ad534b8265c6b67f598f5eb6dcec018b6f22830fee4ffdd76e21e97ff343df', 'linked_sha256': 'a0ad534b8265c6b67f598f5eb6dcec018b6f22830fee4ffdd76e21e97ff343df'}, {'marker': 'topal.policy-inputs.dispatch-library.220', 'object_section': 876, 'address': 44999208, 'bytes': 16, 'executable': False, 'object_sha256': '374708fff7719dd5979ec875d56cd2286f6d3cf7ec317a3b25632aab28ec37bb', 'linked_sha256': '374708fff7719dd5979ec875d56cd2286f6d3cf7ec317a3b25632aab28ec37bb'}]}, {'gs_slot': 992, 'argument': 18, 'entry_stack_offset': 96, 'native_export': 'topal_kernel_structured_process_identity_policy_v1', 'target': 46250784, 'sections': [{'marker': 'topal.policy-inputs.process-identity-library.0', 'object_section': 3, 'address': 46248384, 'bytes': 73, 'executable': True, 'object_sha256': 'cb9c496ef5234e5129bb5b82975701cdc84ec32909a090bae9dd90fe1a7490e4', 'linked_sha256': 'cb9c496ef5234e5129bb5b82975701cdc84ec32909a090bae9dd90fe1a7490e4'}, {'marker': 'topal.policy-inputs.process-identity-library.1', 'object_section': 6, 'address': 46248464, 'bytes': 79, 'executable': True, 'object_sha256': '872017bf481b6c1c97279bcc4acfe82207572829807084c4c2f4dbb1c363ac44', 'linked_sha256': '8972390e2513f1ccd135ec3ebc87475fe764a547957b658700a8c0494050fc20'}, {'marker': 'topal.policy-inputs.process-identity-library.2', 'object_section': 10, 'address': 46248544, 'bytes': 127, 'executable': True, 'object_sha256': '767de80f9cb4df09613ff8c7490087362ebee98e020ec3fb650a2d8943340549', 'linked_sha256': 'e9a8fc3585aa2f5efe9123d6c78a236c8e2f59869429ea999ffab07c61d147aa'}, {'marker': 'topal.policy-inputs.process-identity-library.3', 'object_section': 14, 'address': 46248672, 'bytes': 48, 'executable': True, 'object_sha256': 'b57c05d1d63a3571aa80996828bc652cf377b32827724be3b85363ad5fea8917', 'linked_sha256': 'b57c05d1d63a3571aa80996828bc652cf377b32827724be3b85363ad5fea8917'}, {'marker': 'topal.policy-inputs.process-identity-library.4', 'object_section': 17, 'address': 46248720, 'bytes': 127, 'executable': True, 'object_sha256': '8207954e261e2daab0c96d6e91fdb6a8df841b45e00ca412b5e267bf5d98df4d', 'linked_sha256': '8207954e261e2daab0c96d6e91fdb6a8df841b45e00ca412b5e267bf5d98df4d'}, {'marker': 'topal.policy-inputs.process-identity-library.5', 'object_section': 20, 'address': 46248848, 'bytes': 158, 'executable': True, 'object_sha256': 'f1682ed576c8f19f011cfdca0f8ddf298a7d002fb9a27c4679050fde11d5bc6b', 'linked_sha256': 'f1682ed576c8f19f011cfdca0f8ddf298a7d002fb9a27c4679050fde11d5bc6b'}, {'marker': 'topal.policy-inputs.process-identity-library.6', 'object_section': 23, 'address': 46249008, 'bytes': 212, 'executable': True, 'object_sha256': '569a1b2edad0d2efc6b1be5b07b7f3a5384cf2b6ecc0ee81aeac92171222fe3f', 'linked_sha256': 'd5180bd7440331a2303f0d4ac673b9517ee66fba8cb7e02d59f4bab0b6f5ef86'}, {'marker': 'topal.policy-inputs.process-identity-library.7', 'object_section': 27, 'address': 46249232, 'bytes': 239, 'executable': True, 'object_sha256': '8ee086baa73317c5efd1aa880e03d23b9ff09d07184904a3ba9dfd4a25d12a43', 'linked_sha256': 'e4a81ab23adac0518f9602b0174fca663c0a4136b21681703d544ff71536d530'}, {'marker': 'topal.policy-inputs.process-identity-library.8', 'object_section': 31, 'address': 46249472, 'bytes': 198, 'executable': True, 'object_sha256': '040afd43269a673f1ca481e0806d771988188c46eefdc6c439537f822ca87287', 'linked_sha256': '2b6f1cab4fa8ea8dc97c8cdf5dda673e0d8e43e663c3f826fa987a47dd8d0daf'}, {'marker': 'topal.policy-inputs.process-identity-library.9', 'object_section': 35, 'address': 46249680, 'bytes': 339, 'executable': True, 'object_sha256': 'c6b07ea70d78afdd3f3514c5fab78ed1472067cd30f842a78b33dbae6f7197a6', 'linked_sha256': '222f6832462ddb8b86801eee46d82d07871653ded0cbe7a7d4c6fc45c2c40d5a'}, {'marker': 'topal.policy-inputs.process-identity-library.10', 'object_section': 39, 'address': 46250032, 'bytes': 28, 'executable': True, 'object_sha256': 'd11865a7a2e5c535d1846140ea69e97c0d55ff666cedd2532cfc339bb73cad94', 'linked_sha256': 'd11865a7a2e5c535d1846140ea69e97c0d55ff666cedd2532cfc339bb73cad94'}, {'marker': 'topal.policy-inputs.process-identity-library.11', 'object_section': 42, 'address': 46250064, 'bytes': 11, 'executable': True, 'object_sha256': 'd6cce27151649426900104a94856db51bea1ccc7fe263bfc350f8ed8268d885a', 'linked_sha256': '93b263f84961d177118d185a5cf3cc3852743e5fc748a0a967ce52f76fe3e0ee'}, {'marker': 'topal.policy-inputs.process-identity-library.12', 'object_section': 46, 'address': 46250080, 'bytes': 9, 'executable': True, 'object_sha256': '7fc921d45c8e0dac7d2aa42f26536c8830cd35bfc8bce127eab50a452af67b52', 'linked_sha256': '88b86a417b241417c1798f721ad016528c230d0581511726ebbde22f00737fab'}, {'marker': 'topal.policy-inputs.process-identity-library.13', 'object_section': 50, 'address': 46250096, 'bytes': 71, 'executable': True, 'object_sha256': '141de8359c64c9253f9578e4c49362958761105256757951dc838c4179ce84b3', 'linked_sha256': '591673470afffb6f9e833aab8083ce7c9dcb52bccc04bcf35fb5f5deef50fc36'}, {'marker': 'topal.policy-inputs.process-identity-library.14', 'object_section': 54, 'address': 46250176, 'bytes': 84, 'executable': True, 'object_sha256': 'f480d141a7fb5e79eb3db20e72af8fe7f18d61bd70ea521368051141b7514c19', 'linked_sha256': '4476329a5215b572fd4ed83a22eaccd5dfe9aac7598020177bad315ffad991dd'}, {'marker': 'topal.policy-inputs.process-identity-library.15', 'object_section': 58, 'address': 46250272, 'bytes': 71, 'executable': True, 'object_sha256': 'd2143d7de28d6db2c979f9bd6550a6c063f6a03e9d2d1df1c3fb4a8fb995a863', 'linked_sha256': 'df6cf78c2590dbaf525267cfb6122b8d5c7d66c887835068680536f2f6200ab5'}, {'marker': 'topal.policy-inputs.process-identity-library.16', 'object_section': 62, 'address': 46250352, 'bytes': 134, 'executable': True, 'object_sha256': '22a4dc0ecd0d1a26a9e749ab50dfed198ef31642dd88b9c06bc5d19ff7fe70f7', 'linked_sha256': 'a2f2e887eed8d79bdbc1302b76ab2d11996e5653f44d734d5a3ef5900592bfd2'}, {'marker': 'topal.policy-inputs.process-identity-library.17', 'object_section': 66, 'address': 46250496, 'bytes': 275, 'executable': True, 'object_sha256': '6b980fd0733730a70840214a37004a9ef76b678baf68ffa28839ab90f2a4ddb4', 'linked_sha256': '19674e335b8d88c4c5bee763628059176f9415fa7ca712d792b0f6dcd246d735'}, {'marker': 'topal.policy-inputs.process-identity-library.18', 'object_section': 70, 'address': 46250784, 'bytes': 2464, 'executable': True, 'object_sha256': 'd28ca2c1200a4deb6377667cb54336265758967f0aa6146fbfa09eba342abd12', 'linked_sha256': 'e9f57acc7a6359d916a462109a0f0941201659417fb0762ca000d2965362bb13'}, {'marker': 'topal.policy-inputs.process-identity-library.19', 'object_section': 74, 'address': 44995136, 'bytes': 160, 'executable': False, 'object_sha256': '7d9d6500f5c75d05d56bd4530cc1f3a5abfec4516bb0071b42e4ff3d7d981164', 'linked_sha256': '7d9d6500f5c75d05d56bd4530cc1f3a5abfec4516bb0071b42e4ff3d7d981164'}, {'marker': 'topal.policy-inputs.process-identity-library.20', 'object_section': 75, 'address': 44995296, 'bytes': 16, 'executable': False, 'object_sha256': '374708fff7719dd5979ec875d56cd2286f6d3cf7ec317a3b25632aab28ec37bb', 'linked_sha256': '374708fff7719dd5979ec875d56cd2286f6d3cf7ec317a3b25632aab28ec37bb'}]}], 'sites': [{'offset': 15011, 'address': 45686307, 'return_address': 45686309, 'gs_slot': 984, 'native_export': 'topal_kernel_static_linux_dispatch_policy_v6'}, {'offset': 47427, 'address': 45718723, 'return_address': 45718725, 'gs_slot': 976, 'native_export': 'topal_kernel_static_elf64_header_policy_v1'}, {'offset': 153895, 'address': 45825191, 'return_address': 45825193, 'gs_slot': 952, 'native_export': 'topal_kernel_file_open_policy_v1'}, {'offset': 187631, 'address': 45858927, 'return_address': 45858929, 'gs_slot': 960, 'native_export': 'topal_kernel_sleep_deadline_policy_v1'}, {'offset': 191561, 'address': 45862857, 'return_address': 45862859, 'gs_slot': 992, 'native_export': 'topal_kernel_structured_process_identity_policy_v1'}, {'offset': 226667, 'address': 45897963, 'return_address': 45897965, 'gs_slot': 952, 'native_export': 'topal_kernel_file_open_policy_v1'}], 'scope': 'Exact root relocation/argument/registration-slice and allocated object-to-linked byte binding. ELF flags are not hardware permissions; record call certificates are not independently re-proved here. Native target/permission/state checks, callee/user-pointer effects and complete owner/lifetime/entry qualification remain separate'}, 'callback_header_hex': '7f454c4602010103000000000000000002003e000100000080654a00000000004000000000000000000000000000000000000000400038000b00000000000000', 'kernel_sha256': '04f4f0b01552e5d724187ae841bec2844d00d113eee2ee940adb8304997433c6', 'inventory_sha256': '1408d6f9916bd5a0c37000b98ce1e8c38e6bad793e780366177b9af4e9136711'}
native_sites={'halt': 45795222, 'exits': [45689936, 45691580]}

import gdb
import hashlib
import json
import struct
import traceback

def reg(name): return int(gdb.parse_and_eval('$'+name)) & ((1<<64)-1)
def address(name): return int(gdb.parse_and_eval('(unsigned long long)&'+name))
def memory(pointer, size): return bytes(gdb.selected_inferior().read_memory(pointer, size)) if size else b''
def words(pointer, count): return list(struct.unpack('<'+'Q'*count, memory(pointer, count*8)))
def word(pointer): return words(pointer, 1)[0]
def signed(value): return value if value < (1<<63) else value-(1<<64)
def state(): return words(address('topal_x86_frame_mm_state_v1'), 4)
def prefix(pages): return hashlib.sha256(memory(0x5f7000, pages*4096)).hexdigest()
def credentials_pointer():
    return word(context+920)+96 if memory(context+896,1)[0] == 1 else 0x7fff18
def file_controls():
    return (memory(context+472,152)+memory(context+736,160)).hex()

context = address('topal_bootstrap_storage') + 512
current = pending = startup_current = bridge_current = None
component_returns = {}
syscall_returns = {}
observed = dict(passed=False, executions=[], startup_executions=[], file_executions=[], shell_executions=[], command_executions=[], poll_executions=[], bridge_returns=[])
callee = ['rbx', 'rbp', 'r12', 'r13', 'r14', 'r15']

class Checked(gdb.Breakpoint):
    def stop(self):
        try:
            return self.observe()
        except Exception:
            traceback.print_exc()
            observed['observer_failure'] = traceback.format_exc()
            observed['active_startup'] = startup_current
            observed['active_bridge'] = bridge_current
            with open(observation_path, 'w') as handle:
                json.dump(observed, handle, indent=2)
            return True

class SourceReturned(Checked):
    def observe(self):
        global current
        self.enabled = False
        assert current and pending is None and reg('cs') & 3 == 3
        assert reg('rax') == 0, ('source case failed', reg('rax'), current)
        assert reg('rsp') == current['stack']+8
        assert {name:reg(name) for name in callee} == current['callee']
        after = state()
        assert after[1:3] == current['before'][1:3], (after, current['before'])
        assert prefix(after[2]) == current['prefix_before']
        assert word(context+904) == current['snapshot']
        assert hashlib.sha256(memory(current['snapshot']+0x9e00,512)).hexdigest() == current['command_before']
        if component.get('mode') == 'exec':
            return exec_returned(current,after)
        if component.get('mode') == 'signals':
            return signal_returned(current,after)
        if component.get('mode') == 'pipes':
            return pipe_returned(current,after)
        if component.get('mode') == 'storage':
            return storage_returned(current,after)
        if component.get('mode') == 'serial':
            return serial_returned(current,after)
        calls = current['calls']
        assert [row['number'] for row in calls] == [12,12,63,63,63,63,63,63,63,1,1,1,1,12], calls
        original = current['before'][1]
        expected_results = [original,0x63a000,0,0]+[-14]*5+[0,-9,-22,-14,original]
        assert [row['result'] for row in calls] == expected_results, calls
        assert [row['args'][0] for row in calls[2:9]] == [
            0x5ffff8,0x638ff8,0x639fff,0x63a000,(1<<64)-1,0x400000,0x800000]
        assert calls[0]['args'][0] == 0 and calls[1]['args'][0] == 0x63a000
        assert [row['args'] for row in calls[9:13]] == [
            [1,(1<<64)-1,0],[0,(1<<64)-1,0],
            [1,(1<<64)-1,257],[1,(1<<64)-1,1]]
        assert calls[-1]['args'][0] == original
        current['after'] = after
        observed['executions'].append(current)
        print('SOURCE_USER_CASES_PASS', len(observed['executions']), len(calls), flush=True)
        current = None
        return False

class SourceEntered(Checked):
    def observe(self):
        global current
        assert current is None and pending is None
        assert reg('cs') & 3 == 3 and reg('cr0') & (1<<16)
        stack = reg('rsp')
        assert 0x7fe000 <= stack < 0x800000 and stack % 16 == 8
        return_site = word(stack)
        assert 0x4e0000 <= return_site < 0x4e8000
        before = state()
        assert before[1] == 0x5f7000+before[2]*4096
        snapshot = word(context+904)
        assert snapshot
        current = dict(stack=stack, before=before, prefix_before=prefix(before[2]),
                       callee={name:reg(name) for name in callee}, calls=[],
                       snapshot=snapshot,
                       command_before=hashlib.sha256(memory(snapshot+0x9e00,512)).hexdigest())
        if component.get('mode') == 'signals':
            current['table_before_sha256'] = hashlib.sha256(memory(word(context+928),8192)).hexdigest()
        if component.get('mode') == 'pipes':
            current['pipe_table_before_sha256'] = hashlib.sha256(memory(word(context+928),8192)).hexdigest()
            current['_pipe_initial'] = memory(word(context+928)+PIPE_BASE,PIPE_BYTES).hex()
        if component.get('mode') == 'storage':
            current['storage_table_before_sha256'] = hashlib.sha256(memory(word(context+928),8192)).hexdigest()
            current['_storage_initial'] = memory(word(context+928)+STORAGE_BASE,STORAGE_BYTES).hex()
        if component.get('mode') == 'serial':
            current['serial_table_before_sha256'] = hashlib.sha256(memory(word(context+928),8192)).hexdigest()
            current['serial_received']=[]
            current['serial_markers']=0
        if return_site not in component_returns:
            component_returns[return_site] = SourceReturned('*'+hex(return_site), internal=True)
        component_returns[return_site].enabled = True
        return False

class SyscallReturned(Checked):
    def observe(self):
        global pending
        self.enabled = False
        assert (current or startup_current) and pending and reg('cs') & 3 == 3
        pending['result'] = signed(reg('rax'))
        pending['phase_after'] = memory(context+104,1)[0]
        if current and component.get('mode') == 'exec' and pending['number'] == 59:
            observe_exec_result(pending)
        if current and component.get('mode') == 'signals' and pending['number'] in [13,14]:
            observe_signal_result(pending)
        if current and component.get('mode') == 'pipes' and pending['number'] != 12:
            pipe_result(pending)
        if current and component.get('mode') == 'storage' and pending['number'] != 12:
            storage_result(pending)
        if current and component.get('mode') == 'serial' and pending['number'] != 12:
            serial_result(pending)
        if startup_current and startup_current.get('kind') == 'poll':
            observe_poll_result(pending)
        if startup_current and startup_current.get('kind') in ['shell','command']:
            observe_shell_result(pending)
        if startup_current and startup_current.get('kind') == 'file':
            n = pending['number']
            result = pending['result']
            if (n in [5,262] and result == 0) or (n == 217 and result > 0):
                pointer = pending['args'][2 if n == 262 else 1]
                pending['output_hex'] = memory(pointer,result if n == 217 else 144).hex()
        if pending['result'] in [-14,-22] or pending['number'] == 12:
            assert pending['phase_after'] == pending['phase_before'], pending
        if startup_current and startup_current.get('kind') == 'file' and pending['result'] == -14:
            assert file_controls() == pending['controls_before'], pending
        (current or startup_current)['calls'].append(pending)
        pending = None
        return False

class SyscallEntered(Checked):
    def __init__(self, specification, count):
        super().__init__(specification, internal=True)
        self.count = count

    def observe(self):
        global pending
        assert (current or startup_current) and pending is None and reg('cs') & 3 == 3
        pending = dict(number=reg('rdi'),
                       args=[reg(name) for name in ['rsi','rdx','rcx','r8'][:self.count]],
                       phase_before=memory(context+104,1)[0])
        if current and component.get('mode') == 'exec' and pending['number'] == 59:
            observe_exec_request(pending)
        if current and component.get('mode') == 'signals' and pending['number'] in [13,14]:
            observe_signal_request(pending)
        if current and component.get('mode') == 'pipes' and pending['number'] != 12:
            pipe_request(pending)
        if current and component.get('mode') == 'storage' and pending['number'] != 12:
            storage_request(pending)
        if current and component.get('mode') == 'serial' and pending['number'] != 12:
            serial_request(pending)
        if startup_current and startup_current.get('kind') == 'poll':
            observe_poll_request(pending)
        if startup_current and startup_current.get('kind') in ['shell','command']:
            observe_shell_request(pending)
        if startup_current and startup_current.get('kind') == 'file':
            pending['controls_before'] = file_controls()
            if pending['number'] in [257,262] and pending['args'][1] == 0x637ff8:
                data = memory(0x637ff8,64)
                pending['pathname_hex'] = data[:data.index(0)+1].hex()
        return_site = word(reg('rsp'))
        assert component['base'] <= return_site < component['base']+component['text_bytes']
        if return_site not in syscall_returns:
            syscall_returns[return_site] = SyscallReturned('*'+hex(return_site), internal=True)
        syscall_returns[return_site].enabled = True
        return False

class StartupReturned(Checked):
    def observe(self):
        global startup_current
        self.enabled = False
        row = startup_current
        if row and row.get('kind') == 'poll':
            return poll_returned(row)
        if row and row.get('kind') in ['shell','command']:
            return shell_returned(row)
        if row and row.get('kind') == 'file':
            return file_returned(row)
        assert row and current is None and pending is None
        assert reg('cs') & 3 == 3 and reg('rsp') == row['stack']+8
        assert {name:reg(name) for name in callee} == row['callee']
        assert credentials_pointer() == row['credentials_pointer']
        assert words(row['credentials_pointer'],8) == row['credentials']
        n = row['number']
        expected = -2 if n == 262 else (8 if n == 318 else 0)
        assert signed(reg('rax')) == expected, ('startup source failed', signed(reg('rax')), row)
        after = state()
        assert after[1:3] == row['before'][1:3]
        assert prefix(after[2]) == row['prefix_before']
        calls = row['calls']
        assert calls[0]['number'] == calls[1]['number'] == calls[-1]['number'] == 12
        assert calls[0]['args'][0] == 0 and calls[1]['args'][0] == 0x63a000
        assert [calls[i]['result'] for i in [0,1,-1]] == [row['before'][1],0x63a000,row['before'][1]]
        assert calls[-1]['args'][0] == row['before'][1]
        assert all(call['number'] == n for call in calls[2:-1])
        if n == 262:
            assert [call['result'] for call in calls[2:-1]] == [-14]*4+[-2]
            assert [call['args'][1] for call in calls[2:-1]] == [
                0x63a000,(1<<64)-1,0x800000,0x639fff,0x638ff8]
            assert calls[-2]['args'][2] == (1<<64)-1
            assert memory(row['destination'],144).hex() == row['output_before']
        else:
            errors = calls[2:-2]
            if n == 318:
                assert [call['result'] for call in errors[:3]] == [0,-22,-22]
                assert [call['args'][:3] for call in errors[:3]] == [
                    [(1<<64)-1,0,1],[(1<<64)-1,8,8],[(1<<64)-1,17,1]]
                errors = errors[3:]
            assert [call['result'] for call in errors] == [-14]*5
            index = 3 if n == 302 else (1 if n == 157 else 0)
            assert [call['args'][index] for call in errors] == [
                0x63a000,(1<<64)-1,0x400000,0x800000,0x639fff]
            assert calls[-2]['args'][index] == (0x5ffff8 if n == 302 else 0x638ff8)
            assert calls[-2]['result'] == expected
            output = memory(row['destination'],8 if n == 318 else 16)
            if n == 302: assert output == struct.pack('<QQ',0x800000,(1<<64)-1)
            if n == 157: assert output == b'busybox'+bytes(9)
            row['output_after'] = output.hex()
        row['after'] = after
        row['phase_after'] = memory(context+104,1)[0]
        expected_phase = {302:6,318:6,157:10,262:12}[n]
        # Exec retains an active descriptor/namespace owner. Its general
        # pathname service runs before the sealed startup fallback and does
        # not advance that fallback's phase.
        if n == 262 and row['table_active']:
            expected_phase = row['phase_before']
        assert row['phase_after'] == expected_phase, row
        observed['startup_executions'].append(row)
        print('SOURCE_STARTUP_CASES_PASS', len(observed['startup_executions']), n, len(calls), flush=True)
        startup_current = None
        return False

class StartupEntered(Checked):
    def observe(self):
        global startup_current
        assert current is startup_current is pending is None
        assert reg('cs') & 3 == 3 and reg('cr0') & (1<<16)
        stack = reg('rsp')
        assert 0x7fe000 <= stack < 0x800000 and stack % 16 == 8
        return_site = word(stack)
        assert component['base'] <= return_site < component['base']+component['text_bytes']
        args = [reg(name) for name in ['rsi','rdx','rcx','r8']]
        n = reg('rdi')
        if component.get('mode') == 'poll':
            return poll_entered(n,args,stack,return_site)
        if (n == 0 and (args[0] == 0 or args[2] == 1)) or n in [1,13,16,63,79,80]:
            return shell_entered(n,args,stack,return_site)
        if n in [5,257,217,0] or (n == 262 and args[1] != 0x4a6900):
            return file_entered(n,args,stack,return_site)
        assert n in [302,318,157,262]
        destination = args[{302:3,318:0,157:1,262:2}[n]]
        before = state()
        startup_current = dict(number=n,args=args,stack=stack,before=before,
            prefix_before=prefix(before[2]),callee={name:reg(name) for name in callee},
            calls=[],destination=destination,
            output_before=memory(destination,144 if n == 262 else (8 if n == 318 else 16)).hex(),
            phase_before=memory(context+104,1)[0],
            table_active=word(word(context+928)) == 1,
            credentials_pointer=credentials_pointer(),credentials=words(credentials_pointer(),8))
        assert startup_current['credentials'] == [11,0,12,0,13,0,14,0]
        assert startup_current['phase_before'] == ({302:5,318:6,157:9,262:11}[n])
        if return_site not in component_returns:
            component_returns[return_site] = StartupReturned('*'+hex(return_site), internal=True)
        component_returns[return_site].enabled = True
        return False

class BridgeReturned(Checked):
    def observe(self):
        global bridge_current
        self.enabled = False
        row = bridge_current
        assert row and current is startup_current is pending is None
        assert reg('rsp') == row['stack']+8 and reg('cs') & 3 == 3
        assert {name:reg(name) for name in row['registers']} == row['registers']
        assert credentials_pointer() == row['credentials_pointer']
        assert words(row['credentials_pointer'],8) == [11,0,12,0,13,0,14,0]
        observed['bridge_returns'].append(row)
        bridge_current = None
        return False

class BridgeEntered(Checked):
    def observe(self):
        global bridge_current
        assert current is startup_current is pending is bridge_current is None
        stack = reg('rsp')
        site = word(stack)
        assert 0x4a6100 <= site < 0x4b0000
        bridge_current = dict(stack=stack,return_site=site,credentials_pointer=credentials_pointer(),
            registers={name:reg(name) for name in callee+['rdi','rsi','rdx','r10','r8','r9']})
        if site not in component_returns:
            component_returns[site] = BridgeReturned('*'+hex(site), internal=True)
        component_returns[site].enabled = True
        return False

def file_entered(n,args,stack,return_site):
    global startup_current
    assert word(word(context+928)) == 0, 'bounded file fixture must precede table activation'
    before = state()
    assert before[1:3] == [0x619000,34]
    destination = args[2] if n == 262 else args[1]
    size = 0 if n == 257 else (512 if n == 217 else (128 if n == 0 else 144))
    path = None
    if n in [257,262]:
        data = memory(args[1],64)
        path = data[:data.index(0)+1].hex()
    startup_current = dict(kind='file',number=n,args=args,stack=stack,before=before,
        prefix_before=prefix(before[2]),callee={name:reg(name) for name in callee},calls=[],
        destination=destination,output_before=memory(destination,size).hex(),
        output_size=size,path=path,credentials_pointer=credentials_pointer(),
        credentials=words(credentials_pointer(),8),phase_before=memory(context+104,1)[0],
        controls_before=file_controls())
    assert startup_current['credentials'] == [11,0,12,0,13,0,14,0]
    if return_site not in component_returns:
        component_returns[return_site] = StartupReturned('*'+hex(return_site), internal=True)
    component_returns[return_site].enabled = True
    return False

def file_returned(row):
    global startup_current
    assert current is None and pending is None and reg('cs') & 3 == 3
    assert reg('rsp') == row['stack']+8
    assert {name:reg(name) for name in callee} == row['callee']
    assert credentials_pointer() == row['credentials_pointer']
    assert words(row['credentials_pointer'],8) == row['credentials']
    after = state()
    assert after[1:3] == row['before'][1:3]
    assert prefix(after[2]) == row['prefix_before']
    n = row['number']
    calls = row['calls']
    assert calls[0]['number'] == calls[1]['number'] == calls[-1]['number'] == 12
    assert [calls[i]['args'][0] for i in [0,1,-1]] == [0,0x63a000,0x619000]
    assert [calls[i]['result'] for i in [0,1,-1]] == [0x619000,0x63a000,0x619000]
    actions = calls[2:-1]
    assert all(c['number'] == n for c in actions)
    path = bytes.fromhex(row['path']) if row['path'] else None
    if n in [257,262]:
        assert [c['args'][1] for c in actions[:4]] == [0x63a000,(1<<64)-1,0x800000,0x639fff]
        assert [c['result'] for c in actions[:4]] == [-14]*4
        actions = actions[4:]
        assert all(c['args'][1] == 0x637ff8 and c['pathname_hex'] == row['path'] for c in actions)
    if n == 257:
        expected = -2 if path == b'/dev/tty\0' else 3
        assert len(actions) == 1 and actions[0]['result'] == expected
    elif n == 0:
        expected = 0
        assert [c['args'][1] for c in actions] == [(1<<64)-1,(1<<47)-4096-127,0x63a000]
        assert [c['result'] for c in actions] == [-14,-14,0]
    elif n == 262 and path == b'/var/spool/mail/root\0':
        expected = -2
        assert len(actions) == 1 and actions[0]['result'] == -2 and actions[0]['args'][2] == 0x63a000
    elif n == 217 and len(actions) == 1:
        expected = 0
        assert actions[0]['args'][1] == 0x63a000 and actions[0]['result'] == 0
    else:
        expected = actions[-1]['result']
        assert expected in ([72,80] if n == 217 else [0])
        index = 2 if n == 262 else 1
        assert [c['args'][index] for c in actions[:-1]] == [0x63a000,(1<<64)-1,0x400000,0x800000,0x639fff]
        assert [c['result'] for c in actions[:-1]] == [-14]*5
        actual = 0x639fb0 if n == 217 else (0x5ffff8 if n == 5 and row['args'][0] == 1 else 0x638ff8)
        assert actions[-1]['args'][index] == actual
        size = expected if n == 217 else 144
        assert memory(row['destination'],size).hex() == actions[-1]['output_hex']
        before = bytes.fromhex(row['output_before'])
        assert memory(row['destination']+size,len(before)-size) == before[size:]
        row['output_after'] = memory(row['destination'],size).hex()
    assert signed(reg('rax')) == expected, ('file source failed',signed(reg('rax')),row)
    if 'output_after' not in row:
        assert memory(row['destination'],row['output_size']).hex() == row['output_before']
    row['after'] = after
    row['phase_after'] = memory(context+104,1)[0]
    observed['file_executions'].append(row)
    print('SOURCE_FILE_CASES_PASS',len(observed['file_executions']),n,len(calls),flush=True)
    startup_current = None
    return False


def protected_controls():
    return (memory(context+152,104)+memory(context+416,8)).hex()+file_controls()
def command_state():
    snapshot = word(context+904)
    assert snapshot
    return memory(snapshot+0x9e00,512).hex()
def masked_prefix(pages,pointer,size):
    data = bytearray(memory(0x5f7000,pages*4096))
    left,right=max(pointer,0x5f7000),min(pointer+size,0x5f7000+len(data))
    if right>left: data[left-0x5f7000:right-0x5f7000] = bytes(right-left)
    return hashlib.sha256(data).hexdigest()
def shell_shape(n,args):
    a,b,c,d=args
    if n==16: return False,0,c
    input = n in [1,80] or (n==13 and b!=0)
    size = 390 if n==63 else (32 if n==13 else (2 if n in [79,80] else c))
    pointer = (b if b else c) if n==13 else (a if n in [63,79,80] else b)
    return input,size,pointer

def shell_entered(n,args,stack,return_site):
    global startup_current
    command = n==0 and args[0]==0
    input,size,pointer = (False,args[2],args[1]) if command else shell_shape(n,args)
    before = state()
    assert before[1:3]==[0x619000,34]
    write_pointer,write_size = (0,0) if input else (pointer,size)
    startup_current = dict(kind='command' if command else 'shell',number=n,args=args,
        stack=stack,before=before,callee={name:reg(name) for name in callee},calls=[],
        input=bool(input),size=size,pointer=pointer,
        original_bytes=memory(pointer,size if size else 1).hex(),
        write_pointer=write_pointer,write_size=write_size,
        prefix_before=masked_prefix(before[2],write_pointer,write_size),
        credentials_pointer=credentials_pointer(),credentials=words(credentials_pointer(),8),
        phase_before=memory(context+104,1)[0],queue_before=command_state())
    assert startup_current['credentials']==[11,0,12,0,13,0,14,0]
    if return_site not in component_returns:
        component_returns[return_site]=StartupReturned('*'+hex(return_site),internal=True)
    component_returns[return_site].enabled=True
    return False

def observe_shell_request(call):
    call['protected_before']=protected_controls()
    call['queue_before']=command_state()
    row=startup_current
    if row['kind']=='shell' and row['input'] and call['number']==row['number']:
        index=1 if call['number'] in [1,13] else 0
        pointer=call['args'][index]
        if pointer in [0x638ff8,0x638fff] or component['base']<=pointer<component['base']+component['text_bytes']:
            call['input_hex']=memory(pointer,row['size']).hex()

def observe_shell_result(call):
    row=startup_current
    call['queue_after']=command_state()
    if call['result']==-14:
        assert protected_controls()==call['protected_before'],call
    if row['kind']=='shell' and not row['input'] and call['number']==row['number'] and call['result']>=0:
        n=call['number']
        if n!=16:
            index=2 if n==13 else (0 if n in [63,79] else 1)
            call['output_hex']=memory(call['args'][index],row['size']).hex()

def shell_returned(row):
    global startup_current
    assert current is None and pending is None and reg('cs')&3==3
    assert reg('rsp')==row['stack']+8
    assert {name:reg(name) for name in callee}==row['callee']
    assert credentials_pointer()==row['credentials_pointer']
    assert words(row['credentials_pointer'],8)==row['credentials']
    after=state()
    assert after[1:3]==row['before'][1:3]
    assert masked_prefix(after[2],row['write_pointer'],row['write_size'])==row['prefix_before']
    calls=row['calls']
    assert [calls[i]['number'] for i in [0,1,-1]]==[12,12,12]
    assert [calls[i]['args'][0] for i in [0,1,-1]]==[0,0x63a000,0x619000]
    assert [calls[i]['result'] for i in [0,1,-1]]==[0x619000,0x63a000,0x619000]
    if row['kind']=='command':
        command_returned(row)
    else:
        n,args,size,input=row['number'],row['args'],row['size'],row['input']
        actions=calls[2:-1]
        assert all(call['number']==n for call in actions)
        if n==16:
            expected=-25
            assert len(actions)==1 and actions[0]['args'][2]==(1<<64)-1
            assert memory(row['pointer'],1).hex()==row['original_bytes']
        else:
            index=1 if n in [1,13,0] and (n!=13 or input) else (2 if n==13 else 0)
            bad=[0x63a000,(1<<64)-1,0x800000,0x63b000 if size==1 else 0x639fff]
            if not input: bad.append(0x400000)
            assert [call['args'][index] for call in actions[:-1]]==bad
            assert [call['result'] for call in actions[:-1]]==[-14]*len(bad)
            expected=size if n in [0,1,79] else 0
            good=actions[-1]
            assert good['result']==expected
            if input:
                assert good['input_hex']==row['original_bytes']
                if n==13: assert component['base']<=good['args'][index]<component['base']+component['text_bytes']
                assert memory(row['pointer'],size).hex()==row['original_bytes']
            else:
                assert memory(row['pointer'],size).hex()==good['output_hex']
            row['output_after']=memory(row['pointer'],size).hex()
        assert signed(reg('rax'))==expected,('shell case failed',signed(reg('rax')),row)
        observed['shell_executions'].append(row)
        print('SOURCE_SHELL_CASES_PASS',len(observed['shell_executions']),n,len(calls),flush=True)
    row['after']=after
    row['phase_after']=memory(context+104,1)[0]
    startup_current=None
    return False

def command_returned(row):
    actions=row['calls'][2:-1]
    assert [call['number'] for call in actions]==[0]*10
    count=row['args'][2]
    assert [call['args'][1] for call in actions]==[
        (1<<64)-1,0x63a000,(1<<64)-1,0x400000,0x800000,0x639fff,
        0x638ff8,(1<<64)-1,0x639fff,0x638ff9]
    assert [call['args'][2] for call in actions]==[0]+[count]*5+[1,0,count,count]
    assert [call['result'] for call in actions[:9]]==[0]+[-14]*5+[1,0,-14]
    rest=actions[-1]['result']
    assert rest>0 and signed(reg('rax'))==rest+1
    assert all(call['phase_after']==call['phase_before'] for call in actions[:-1])
    assert actions[0]['queue_before']==actions[0]['queue_after']
    assert actions[1]['queue_after']==actions[2]['queue_after']==actions[3]['queue_after']==actions[4]['queue_after']==actions[5]['queue_after']
    assert actions[6]['queue_after']==actions[7]['queue_after']==actions[8]['queue_after']
    serial=count==23
    def queue(call):
        data=bytes.fromhex(call['queue_after'])
        return data[:256],struct.unpack_from('<QQQ',data,256)
    if serial:
        queued=queue(actions[1]); short=queue(actions[6]); complete=queue(actions[-1])
        assert queued[1][1]==23 and short[1][1]==22 and complete[1][1]==0
        assert short[0][:22]==queued[0][1:23] and short[0][22:]==bytes(234)
        assert complete[0]==bytes(256)
    else:
        assert queue(actions[1])[1][0]==0 and queue(actions[6])[1][0]==1
        assert queue(actions[-1])[1][0]==0
    before=bytes.fromhex(row['original_bytes'])
    total=rest+1
    assert memory(row['pointer']+total,len(before)-total)==before[total:]
    row['delivered_hex']=memory(row['pointer'],total).hex()
    observed['command_executions'].append(row)
    print('SOURCE_COMMAND_CASES_PASS',len(observed['command_executions']),total,len(row['calls']),flush=True)



def exec_protected():
    table = word(context+928)
    assert word(table) == 1
    return dict(table_sha256=hashlib.sha256(memory(table,8192)).hexdigest(),
        registrations=memory(context+152,104).hex(),
        shell=memory(context+416,8).hex(),files=memory(context+736,160).hex(),
        command=memory(word(context+904)+0x9e00,512).hex(),
        snapshot=word(context+904),auxiliary=word(context+920))

def observe_exec_request(call):
    call['protected_before'] = exec_protected()
    assert memory(word(context+904)+0x9800,0x600) == bytes(0x600)
    name,argv,envp = call['args']
    def candidate(address,size):
        # Read only the candidate's actual mapped portion, including code data.
        if component['base'] <= address < component['base']+component['text_bytes']:
            return memory(address,min(size,component['base']+component['text_bytes']-address))
        if 0x5f7000 <= address < 0x63a000:
            return memory(address,min(size,0x63a000-address))
        return b''
    call['pathname_prefix_hex'] = candidate(name,256).hex()
    call['argv_prefix_hex'] = candidate(argv,72).hex()
    call['environment_prefix_hex'] = candidate(envp,136).hex()

def observe_exec_result(call):
    assert call['result'] in [-14,-2,-36,-22,-7],call
    assert exec_protected() == call['protected_before'],call
    assert memory(word(context+904)+0x9800,0x600) == bytes(0x600)
    call['private_staging_zero'] = True
    call['protected_state_unchanged'] = True

def exec_returned(row,after):
    global current
    calls = row['calls']
    assert [c['number'] for c in calls] == [12,12]+[59]*29+[12]
    assert [calls[i]['args'][0] for i in [0,1,-1]] == [0,0x63a000,0x619000]
    assert [calls[i]['result'] for i in [0,1,-1]] == [0x619000,0x63a000,0x619000]
    actions=calls[2:-1]
    expected = [-14]*5+[-2,-36]+[-14]*10+[-22,-7,-7]+[-14]*6+[-7]*3
    assert len(expected)==29
    assert [c['result'] for c in actions] == expected
    assert all(c['private_staging_zero'] and c['protected_state_unchanged'] for c in actions)
    assert [c['args'][0] for c in actions[:5]] == [0,0x63a000,(1<<64)-1,0x800000,0x639fff]
    assert [c['args'][1] for c in actions[7:12]] == [0,0x63a000,(1<<64)-4,0x800000,0x639ffc]
    assert actions[12]['args'][1] == 0x5ffffc
    assert actions[13]['args'][:2] == [0x638ff8,0x637ffc]
    assert component['base'] <= actions[14]['args'][1] < component['base']+component['text_bytes']
    assert actions[16]['args'][1] == 0x639ff0
    assert actions[23]['args'][2] == 0x639ffc
    assert actions[25]['args'][2] == 0x639ff0
    row['after'] = after
    observed['executions'].append(row)
    print('SOURCE_EXEC_CASES_PASS',len(observed['executions']),len(actions),flush=True)
    current = None
    return False



def signal_read(pointer,size):
    if component['base'] <= pointer < component['base']+component['text_bytes']:
        return memory(pointer,min(size,component['base']+component['text_bytes']-pointer))
    if 0x5f7000 <= pointer < 0x63a000:
        return memory(pointer,min(size,0x63a000-pointer))
    if pointer == 0x400000:
        return memory(pointer,size)
    return b''

def signal_writable(pointer,size):
    return 0x5f7000 <= pointer and pointer+size <= 0x63a000

def signal_context():
    return (memory(context+152,104)+memory(context+416,8)+memory(context+736,160)+
            memory(word(context+904)+0x9e00,512)).hex()

def signal_state(table):
    return dict(blocked=struct.unpack_from('<Q',table,32)[0],
        action10=list(struct.unpack_from('<4Q',table,1792+9*32)))

def observe_signal_request(call):
    table=memory(word(context+928),8192)
    assert word(word(context+928)) == 1
    assert table[1408:1664] == bytes(256)
    call['_table_before'] = table.hex()
    call['state_before'] = signal_state(table)
    call['context_before'] = signal_context()
    size=32 if call['number']==13 else 8
    _,new,old,_=call['args']
    call['input_hex'] = signal_read(new,size).hex() if new else ''
    call['old_output_prefix_before'] = signal_read(old,size).hex() if old else ''

def observe_signal_result(call):
    table=bytearray.fromhex(call.pop('_table_before'))
    number=call['number']
    selector,new,old,sigsetsize=call['args']
    size=32 if number==13 else 8
    input_bytes=bytes.fromhex(call['input_hex'])
    expected=0
    output=None
    committed=False
    if sigsetsize!=8: expected=-22
    elif new and len(input_bytes)!=size: expected=-14
    elif number==13:
        if not 1<=selector<=64 or (new and selector in [9,19]): expected=-22
        else:
            at=1792+(selector-1)*32
            output=bytes(table[at:at+32])
            if new:
                handler,flags,restorer,mask=struct.unpack('<4Q',input_bytes)
                table[at:at+32]=struct.pack('<4Q',handler,flags&0xdc000807,restorer,mask&~0x40100)
                committed=True
    else:
        output=bytes(table[32:40])
        if new:
            if selector not in [0,1,2]: expected=-22
            else:
                requested=struct.unpack('<Q',input_bytes)[0]&~0x40100
                previous=struct.unpack('<Q',output)[0]
                value=[previous|requested,previous&~requested,requested][selector]
                table[32:40]=struct.pack('<Q',value)
                committed=True
    if expected==0 and old:
        if signal_writable(old,size):
            assert memory(old,size)==output,call
            call['output_hex']=output.hex()
        else: expected=-14
    assert call['result']==expected,call
    actual=memory(word(context+928),8192)
    assert actual==bytes(table),call
    assert signal_context()==call['context_before'],call
    if expected<0 and old:
        assert signal_read(old,size).hex()==call['old_output_prefix_before'],call
    assert actual[1408:1664]==bytes(256)
    call['state_after']=signal_state(actual)
    call['committed']=committed
    call['complete_table_verified']=True
    call['private_staging_zero']=True

def signal_returned(row,after):
    global current
    calls=row['calls']
    assert [c['number'] for c in calls]==[12,12]+[13]*23+[14]*23+[12]
    assert [calls[i]['result'] for i in [0,1,-1]]==[0x619000,0x63a000,0x619000]
    actions=calls[2:-1]
    assert all(c['complete_table_verified'] and c['private_staging_zero'] for c in actions)
    assert sum(c['result']==-14 for c in actions)==22
    assert sum(c['result']==-22 for c in actions)==6
    assert sum(c['result']==0 for c in actions)==18
    assert sum(c['committed'] for c in actions)==10
    assert sum(c['committed'] and c['result']==-14 for c in actions)==2
    assert hashlib.sha256(memory(word(context+928),8192)).hexdigest()==row['table_before_sha256']
    row['after']=after
    observed['executions'].append(row)
    print('SOURCE_SIGNAL_CASES_PASS',len(observed['executions']),len(actions),flush=True)
    current=None
    return False



def poll_controls():
    return dict(registrations=memory(context+152,104).hex(),header=memory(context+416,8).hex(),
        stat_staging=memory(context+472,152).hex(),file=memory(context+736,160).hex(),queue=command_state())

def poll_bytes(pointer,size):
    if component['base']<=pointer<component['base']+component['text_bytes']:
        return memory(pointer,min(size,component['base']+component['text_bytes']-pointer))
    if 0x5f7000<=pointer<0x63a000:
        return memory(pointer,min(size,0x63a000-pointer))
    if pointer==0x400000: return memory(pointer,size)
    return b''

def poll_entered(n,args,stack,return_site):
    global startup_current
    assert n==0 and args[0]==3 and args[2]==1
    index=len(observed['poll_executions'])
    assert memory(context+416,1)==b'\x01'
    assert memory(context+888,4)==bytes([22+index,index,0,0])
    assert word(word(context+928))==0
    before=state()
    assert before[1:3]==[0x619000,34]
    startup_current=dict(kind='poll',number=n,args=args,stack=stack,before=before,
        index=index,callee={name:reg(name) for name in callee},calls=[],
        prefix_before=prefix(before[2]),credentials_pointer=credentials_pointer(),
        credentials=words(credentials_pointer(),8),controls_before=poll_controls())
    if return_site not in component_returns:
        component_returns[return_site]=StartupReturned('*'+hex(return_site),internal=True)
    component_returns[return_site].enabled=True
    return False

def observe_poll_request(call):
    call['controls_before']=poll_controls()
    n,args=call['number'],call['args']
    if n==7: call['input_hex']=poll_bytes(args[0],8).hex()
    if n==0: call['output_before']=poll_bytes(args[1],1).hex()

def observe_poll_result(call):
    expected=dict(call['controls_before'])
    fields=bytearray.fromhex(expected['file'])
    n,args,result=call['number'],call['args'],call['result']
    if n==7:
        assert args[1:3]==[1,(1<<64)-1]
        if result==1:
            assert struct.unpack('<iH',bytes.fromhex(call['input_hex'])[:6])==(3,1)
            after=memory(args[0],8)
            assert after[:6]==bytes.fromhex(call['input_hex'])[:6] and after[6:]==b'\x01\0'
            call['output_hex']=after.hex()
            fields[891-736]=1
        else:
            assert result==-14
            assert poll_bytes(args[0],8).hex()==call['input_hex']
    elif n==0:
        assert args[0]==3 and args[2]==1
        if result==1:
            assert fields[891-736]==1
            value=bytes([b'\x7fELF'[fields[889-736]]])
            assert memory(args[1],1)==value
            call['output_hex']=value.hex()
            fields[888-736]+=1
            fields[889-736]+=1
            fields[891-736]=0
        else:
            assert result==-14 and poll_bytes(args[1],1).hex()==call['output_before']
    else: assert n==12
    expected['file']=fields.hex()
    actual=poll_controls()
    assert actual==expected,call
    call['controls_after']=actual

def poll_returned(row):
    global startup_current
    assert current is pending is None and signed(reg('rax'))==1
    assert reg('rsp')==row['stack']+8 and reg('cs')&3==3
    assert {name:reg(name) for name in callee}==row['callee']
    assert words(row['credentials_pointer'],8)==row['credentials']
    assert state()[1:3]==row['before'][1:3] and prefix(34)==row['prefix_before']
    calls=row['calls']
    assert [c['number'] for c in calls]==[12,12]+[7]*8+[0]*5+[12]
    assert [calls[i]['result'] for i in [0,1,-1]]==[0x619000,0x63a000,0x619000]
    assert [c['result'] for c in calls[2:-1]]==[-14]*5+[1,-14,1]+[-14]*4+[1]
    assert memory(row['args'][1],1)==bytes([b'\x7fELF'[row['index']]])
    assert memory(context+888,4)==bytes([23+row['index'],row['index']+1,0,0])
    row['after']=state()
    observed['poll_executions'].append(row)
    print('SOURCE_POLL_CASES_PASS',len(observed['poll_executions']),len(calls),flush=True)
    startup_current=None
    return False



extent_first_reach = {}

class ExtentFirstReach(Checked):
    def __init__(self, row):
        super().__init__('*'+hex(extent_inventory['address']+row['offset']), internal=True)
        self.row = row

    def observe(self):
        self.enabled = False
        assert reg('cs') & 3 == 0 and reg('cr0') & (1<<16)
        offset = self.row['offset']
        assert offset not in extent_first_reach
        extent_first_reach[offset] = dict(offset=offset, target=self.row['target'],
            candidate=reg('r8'), length=reg('r9'), phase=memory(context+104,1)[0])
        return False

def extent_coverage_complete():
    if extent_inventory is None:
        return
    callback_coverage_complete()
    calls = extent_inventory['calls']
    assert extent_first_reach and set(extent_first_reach) <= {row['offset'] for row in calls}
    observed['extent_call_coverage'] = dict(
        observation_passed=True, total_sites=len(calls), reached_sites=len(extent_first_reach),
        reached=[extent_first_reach[offset] for offset in sorted(extent_first_reach)],
        unreached=[row for row in calls if row['offset'] not in extent_first_reach],
        kernel_sha256=extent_inventory['kernel_sha256'],
        inventory_sha256=extent_inventory['inventory_sha256'],
        scope='First reach before helper invocation in this selected execution only; no successful-admission, return, every-length/error-path, direct-access/dominance or production qualification claim')

def install_extent_coverage():
    if extent_inventory is not None:
        install_callback_observation()
        assert address(extent_inventory['function']) == extent_inventory['address']
        for row in extent_inventory['calls']:
            ExtentFirstReach(row)

import hashlib

callback_active = None
callback_registered = None
callback_counts = {}
callback_samples = {}
callback_live_sections = {}
callback_callee_registers = ['rbx','rbp','r12','r13','r14','r15']

def callback_translation(pointer):
    assert not (reg('cr4') & (1<<12))
    table = reg('cr3') & 0x000ffffffffff000
    path = []
    user, writable, noexecute = True, True, False
    for shift in [39,30,21,12]:
        assert table % 4096 == 0 and table < 0x40000000
        entry = word(table + ((pointer>>shift)&511)*8)
        assert entry & 1
        user = user and bool(entry&4)
        writable = writable and bool(entry&2)
        noexecute = noexecute or bool(entry&(1<<63))
        path.append(dict(shift=shift,table=table,entry=entry))
        if shift == 12 or (shift in [30,21] and entry&128):
            physical = (entry & (0x000fffffffffffff & ~((1<<shift)-1))) | (pointer&((1<<shift)-1))
            assert physical == pointer and not user and not noexecute
            return dict(path=path,physical=physical,effective_user=user,
                        effective_write=writable,effective_execute=not noexecute)
        table = entry & 0x000ffffffffff000
    raise AssertionError('missing translation leaf')

class CallbackRegistration(Checked):
    def observe(self):
        global callback_registered
        self.enabled = False
        assert callback_registered is None and reg('cs')&3 == 0
        assert word(reg('rsp')+8) == context
        rows = []
        for row in extent_inventory['callback_bindings']['callbacks']:
            actual = word(reg('rsp')+row['entry_stack_offset'])
            assert actual == row['target'] == address(row['native_export'])
            rows.append(dict(gs_slot=row['gs_slot'],argument=row['argument'],target=actual))
        callback_registered = rows
        return False

class CallbackReturned(Checked):
    def __init__(self, site):
        super().__init__('*'+hex(site['return_address']),internal=True)
        self.site = site

    def observe(self):
        global callback_active
        assert callback_active is not None and callback_active['site']==self.site['offset']
        before = callback_active
        assert reg('cs')&3 == 0 and reg('cr0')&(1<<16)
        assert reg('cr3') == before['cr3'] and reg('rsp') == before['rsp']
        assert [reg(r) for r in callback_callee_registers] == before['callee_registers']
        assert memory(context+296,64) == before['captured']
        assert memory(context+952,56) == before['registry']
        assert memory(before['table'],8192) == before['table_bytes']
        assert memory(context+104,1)[0] == before['phase']
        slot = self.site['gs_slot']
        callback_counts[self.site['offset']] = callback_counts.get(self.site['offset'],0)+1
        if self.site['offset'] not in callback_samples:
            callback_samples[self.site['offset']] = dict(
                offset=self.site['offset'],gs_slot=slot,target=before['target'],
                arguments=before['arguments'],result=reg('rax'),
                supervisor_translation=before['translation'],
                callee_registers_preserved=True,captured_arguments_preserved=True,
                callback_registry_preserved=True,complete_process_table_preserved=True)
        callback_active = None
        return False

class CallbackEntered(Checked):
    def __init__(self, site, row):
        super().__init__('*'+hex(site['address']),internal=True)
        self.site, self.row = site, row

    def observe(self):
        global callback_active
        assert callback_registered is not None and callback_active is None
        assert reg('cs')&3 == 0 and reg('cr0')&(1<<16) and not (reg('eflags')&0x200)
        row, site = self.row, self.site
        assert reg('rax') == word(context+row['gs_slot']) == row['target']
        assert reg('rsp') % 16 == 0
        translation = callback_translation(reg('rax'))
        if row['gs_slot'] not in callback_live_sections:
            checked = []
            for section in row['sections']:
                actual = hashlib.sha256(memory(section['address'],section['bytes'])).hexdigest()
                assert actual == section['linked_sha256']
                checked.append(dict(address=section['address'],bytes=section['bytes'],sha256=actual))
            callback_live_sections[row['gs_slot']] = checked
        if row['gs_slot'] == 952:
            # The complete file-open plan packs eight Boolean facts. Anonymous
            # creation has its own exact write/mutable/create combination.
            assert reg('rdi') < 256
            if site['offset'] == 226667:
                assert reg('rdi') == 1 | 8 | 32
        elif row['gs_slot'] == 984:
            assert reg('rdi') <= 0xffffffff
        elif row['gs_slot'] == 976:
            table = word(context+928)
            assert word(table)==1 and word(table+48)==1 and word(table+24)>=64
            assert reg('rdi') == word(table+16)
            assert [reg(r) for r in ['rsi','rdx','rcx']] == [64,62,16]
            assert memory(reg('rdi'),64).hex() == extent_inventory['callback_header_hex']
        elif row['gs_slot'] == 992:
            assert reg('rsi') == word(context+1000) and reg('rsi')
            assert reg('rdx') == word(context+256) == context and reg('rcx') == row['target']
        table = word(context+928)
        assert table
        callback_active = dict(site=site['offset'],target=reg('rax'),cr3=reg('cr3'),rsp=reg('rsp'),
            callee_registers=[reg(r) for r in callback_callee_registers],
            arguments=[reg(r) for r in ['rdi','rsi','rdx','rcx','r8','r9']],
            captured=memory(context+296,64),registry=memory(context+952,56),
            table=table,table_bytes=memory(table,8192),phase=memory(context+104,1)[0],
            translation=translation)
        return False

def install_callback_observation():
    if extent_inventory is None:
        return
    bindings = extent_inventory['callback_bindings']
    assert bindings['passed'] and bindings['kernel_sha256'] == extent_inventory['kernel_sha256']
    rows = {row['gs_slot']:row for row in bindings['callbacks']}
    CallbackRegistration('*'+hex(extent_inventory['address']),internal=True)
    for site in bindings['sites']:
        CallbackEntered(site,rows[site['gs_slot']])
        CallbackReturned(site)

def callback_coverage_complete():
    if extent_inventory is None:
        return
    assert callback_registered is not None and callback_active is None and callback_counts
    sites = extent_inventory['callback_bindings']['sites']
    observed['callback_observation'] = dict(passed=True,registered=callback_registered,
        counts=[dict(offset=offset,count=callback_counts[offset]) for offset in sorted(callback_counts)],
        samples=[callback_samples[offset] for offset in sorted(callback_samples)],
        live_sections=[dict(gs_slot=slot,sections=callback_live_sections[slot]) for slot in sorted(callback_live_sections)],
        unreached=[site for site in sites if site['offset'] not in callback_counts],
        scope='Actual callback registration/targets, supervisor-executable mapping, first-role live bytes and observed ABI/private-state preservation; not every callee path, hardware read-only protection, arbitrary pointer authority, complete memory effects or lifetime proof')



PIPE_BASE=0x3e000
PIPE_BYTES=0x5000

def pipe_context():
    return (memory(context+152,104)+memory(context+416,8)+memory(context+736,160)+
            memory(word(context+904)+0x9e00,512)).hex()

def pipe_visible(pointer,size):
    for start,end in [(component['base'],component['base']+component['text_bytes']),
                      (0x5f7000,0x63a000),(0x400000,0x401000)]:
        if start<=pointer<end:
            return memory(pointer,min(size,end-pointer))
    return b''

def pipe_request(call):
    table=word(context+928)
    assert word(table)==1 and memory(table+1408,256)==bytes(256)
    call['_table_before']=memory(table,8192).hex()
    call['_pipe_before']=memory(table+PIPE_BASE,PIPE_BYTES).hex()
    call['context_before']=pipe_context()
    n=call['number']
    args=call['args']
    if n in [22,293]: pointer,size=args[0],8
    elif n==16: pointer,size=args[2],4
    else: pointer,size=args[1],min(args[2],16)
    if n!=3:
        call['buffer_before']=pipe_visible(pointer,size).hex()
        call['buffer_pointer']=pointer
        call['window_pointer']=pointer-1
        call['window_before']=pipe_visible(pointer-1,size+2).hex()
    if n in [0,1,3,16]:
        description=word(table+64+args[0]*16)
        assert 1<=description<=16
        call['description_index']=description
        call['description_before']=words(table+320+(description-1)*64,8)

def pipe_result(call):
    table=word(context+928)
    before_table=bytearray.fromhex(call.pop('_table_before'))
    before=bytearray.fromhex(call.pop('_pipe_before'))
    after=memory(table+PIPE_BASE,PIPE_BYTES)
    n,result=call['number'],call['result']
    args=call['args']
    expected_table=bytearray(before_table)
    expected=bytearray(before)
    output=b''
    if result<0:
        assert result in [-14,-11]
        if n!=3:
            assert pipe_visible(call['buffer_pointer'],len(bytes.fromhex(call['buffer_before']))).hex()==call['buffer_before']
    elif n in [22,293]:
        assert result==0
        free=[fd for fd in range(16) if struct.unpack_from('<Q',before_table,64+fd*16)[0]==0]
        pair=list(struct.unpack('<II',memory(args[0],8)))
        assert pair==free[:2]
        generation=struct.unpack_from('<Q',before,512)[0]+1
        index=next(i for i in range(4) if before[i*128:i*128+128]==bytes(128))
        current['pipe_index']=index
        call['pair']=pair
        call['generation']=generation
        call['object_index']=index
        descs=[struct.unpack_from('<Q',memory(table,8192),64+fd*16)[0] for fd in pair]
        assert descs[0]!=descs[1] and all(1<=d<=16 for d in descs)
        for endpoint,(fd,description) in enumerate(zip(pair,descs)):
            assert before_table[320+(description-1)*64:320+description*64]==bytes(64)
            struct.pack_into('<QQ',expected_table,64+fd*16,description,0)
            struct.pack_into('<8Q',expected_table,320+(description-1)*64,
                             5+endpoint,1,0x800|endpoint,0,generation,0,0,0)
        record=list(struct.unpack_from('<16Q',after,index*128))
        assert record[:9]==[1,generation,*descs,0,0,0,0,0o10600]
        assert record[9]==record[10]==record[11]==record[12]>0 and record[13:]==[0]*3
        expected[index*128:index*128+128]=after[index*128:index*128+128]
        struct.pack_into('<Q',expected,512,generation)
        output=struct.pack('<II',*pair)
    else:
        index=current['pipe_index']
        at=index*128
        ring=0x1000+index*4096
        cursor,count=struct.unpack_from('<QQ',before,at+32)
        if n==16:
            assert result==0 and args[1]==0x541b
            output=struct.pack('<I',count)
            assert memory(args[2],4)==output
        elif n in [0,1] and result>0:
            assert result<=args[2]
            if n==0:
                assert result==min(args[2],count)
                output=bytes(before[ring+(cursor+i)%4096] for i in range(result))
                assert memory(args[1],result)==output
                for i in range(result): expected[ring+(cursor+i)%4096]=0
                struct.pack_into('<QQ',expected,at+32,(cursor+result)%4096,count-result)
                timestamp_offsets=[72]
                call['output_hex']=output.hex()
            else:
                assert result==min(args[2],4096-count)
                data=bytes.fromhex(call['buffer_before'])
                assert len(data)>=result
                for i in range(result): expected[ring+(cursor+count+i)%4096]=data[i]
                struct.pack_into('<Q',expected,at+40,count+result)
                timestamp_offsets=[80,88]
            for offset in timestamp_offsets:
                assert struct.unpack_from('<Q',after,at+offset)[0]>0
                expected[at+offset:at+offset+8]=after[at+offset:at+offset+8]
        elif n==3:
            assert result==0 and call['description_before'][1]==1
            description=call['description_index']
            expected_table[64+args[0]*16:80+args[0]*16]=bytes(16)
            expected_table[320+(description-1)*64:320+description*64]=bytes(64)
            offset=16 if call['description_before'][0]==5 else 24
            struct.pack_into('<Q',expected,at+offset,0)
            if struct.unpack_from('<QQ',expected,at+16)==(0,0):
                expected[at:at+128]=bytes(128)
                expected[ring:ring+4096]=bytes(4096)
        else:
            assert n==0 and result==0 and count==0
    assert after==bytes(expected), (n,result,'FIFO bytes changed outside permitted transition')
    assert memory(table,8192)==bytes(expected_table), (n,result,'descriptor/staging change')
    assert pipe_context()==call['context_before']
    if n!=3:
        window=bytearray.fromhex(call['window_before'])
        if output:
            assert 1+len(output)<=len(window)
            window[1:1+len(output)]=output
        assert pipe_visible(call['window_pointer'],len(window))==bytes(window), (n,result,'user copy window')
        call['copy_window_verified']=True
    call['complete_fifo_state_verified']=True
    call['complete_table_verified']=True
    call['pipe_state_after_sha256']=hashlib.sha256(after).hexdigest()

def pipe_returned(row,after):
    global current
    calls=row['calls']
    assert len(calls)==36
    assert {n:sum(c['number']==n for c in calls) for n in [12,293,16,1,0,3]}=={
        12:3,293:6,16:10,1:6,0:9,3:2}
    actions=[c for c in calls if c['number']!=12]
    assert sum(c['result']==-14 for c in actions)==19
    assert sum(c['result']==-11 for c in actions)==1
    assert all(c['complete_fifo_state_verified'] and c['complete_table_verified'] for c in actions)
    table=word(context+928)
    assert hashlib.sha256(memory(table,8192)).hexdigest()==row['pipe_table_before_sha256']
    shared=memory(table+PIPE_BASE,PIPE_BYTES)
    old=bytearray.fromhex(row.pop('_pipe_initial'))
    struct.pack_into('<Q',old,512,struct.unpack_from('<Q',old,512)[0]+1)
    assert shared==bytes(old), 'FIFO object/ring not retired exactly; only generation history may advance'
    row['after']=after
    row['pipe_retirement_verified']=True
    observed['executions'].append(row)
    print('SOURCE_PIPE_CASES_PASS',len(observed['executions']),len(calls),flush=True)
    current=None
    return False



STORAGE_BASE=0x43000
STORAGE_BYTES=0x9000

def storage_word(data,offset): return struct.unpack_from('<Q',data,offset)[0]
def storage_put(data,offset,value): struct.pack_into('<Q',data,offset,value)

def storage_request(call):
    table=word(context+928)
    assert word(table)==1 and memory(table+1408,256)==bytes(256)
    call['_table_before']=memory(table,8192).hex()
    call['_storage_before']=memory(table+STORAGE_BASE,STORAGE_BYTES).hex()
    call['context_before']=pipe_context()
    n,args=call['number'],call['args']
    if n==257:
        raw=pipe_visible(args[1],64)
        path=raw[:raw.index(0)+1]
        call['path_hex']=path.hex()
        pointer,size=args[1],len(path)
    elif n in [0,1,17,18,217]:
        pointer,size=args[1],min(args[2],48)
    else:
        pointer=size=None
    if pointer is not None:
        call['buffer_before']=pipe_visible(pointer,size).hex()
        call['buffer_pointer']=pointer
        call['window_pointer']=pointer-1
        call['window_before']=pipe_visible(pointer-1,size+2).hex()
    if n!=257:
        index=word(table+64+args[0]*16)
        assert 1<=index<=16
        call['description_index']=index
        call['description_before']=words(table+320+(index-1)*64,8)

def storage_result(call):
    table=word(context+928)
    before_table=bytearray.fromhex(call.pop('_table_before'))
    before=bytearray.fromhex(call.pop('_storage_before'))
    actual=memory(table+STORAGE_BASE,STORAGE_BYTES)
    expected=bytearray(before)
    expected_table=bytearray(before_table)
    n,args,result=call['number'],call['args'],call['result']
    output=b''
    if result<0:
        assert result in [-14,-22]
        if 'buffer_pointer' in call:
            raw=bytes.fromhex(call['buffer_before'])
            assert pipe_visible(call['buffer_pointer'],len(raw))==raw
    elif n==257:
        free=[i for i in range(16) if storage_word(before_table,64+i*16)==0]
        assert result==free[0]
        index=word(table+64+result*16)
        assert 1<=index<=16
        at=320+(index-1)*64
        assert before_table[at:at+64]==bytes(64)
        path=bytes.fromhex(call['path_hex'])
        flags=args[2]
        if flags==0x410002:
            assert path==b'/tmp\0'
            identity=storage_word(before,0)
            slot=next(i for i in range(4) if before[256+i*128:384+i*128]==bytes(128))
            object_at=256+slot*128
            record=list(struct.unpack_from('<16Q',actual,object_at))
            mode=0o100000|(args[3]&0o7777&~storage_word(before_table,1664))
            assert record[:4]==[1,identity,0,mode]
            assert record[4]==record[5]==record[6]>0
            assert record[7:]==[0,1]+[0]*7
            expected[object_at:object_at+128]=actual[object_at:object_at+128]
            storage_put(expected,0,identity+1)
            kind=7
            call['object_identity']=identity
            call['object_slot']=slot
        else:
            assert (path,flags) in [(b'/bin/busybox\0',0),(b'/tmp\0',0x10000)]
            kind,identity=(3,5) if flags==0 else (4,6)
        status=(flags&~(0x80000|0x100|0x40|0x80|0x200))|0x8000
        struct.pack_into('<8Q',expected_table,at,kind,1,status,0,identity,0,0,0)
        struct.pack_into('<QQ',expected_table,64+result*16,index,0)
        call['fresh_description_verified']=True
    else:
        desc=call['description_before']
        desc_at=320+(call['description_index']-1)*64
        kind,_,_,position,identity,*_=desc
        if n==8:
            assert args[2] in [0,1]
            expected_position=args[1]+(position if args[2]==1 else 0)
            assert result==expected_position
            storage_put(expected_table,desc_at+24,result)
        elif n==217:
            assert kind==4 and identity==6 and position==0 and result==48
            records=bytearray(48)
            for i,(inode,name) in enumerate([(6,b'.\0'),(3,b'..\0')]):
                struct.pack_into('<QQHB',records,i*24,inode,i+1,24,4)
                records[i*24+19:i*24+19+len(name)]=name
            output=bytes(records)
            cursor=next((i+2 for i in range(4) if storage_word(before,64+i*16)),6)
            storage_put(expected_table,desc_at+24,cursor)
        elif n==3:
            assert result==0 and desc[1]==1
            expected_table[64+args[0]*16:80+args[0]*16]=bytes(16)
            expected_table[desc_at:desc_at+64]=bytes(64)
            if kind==7:
                object_at=next(256+i*128 for i in range(4)
                    if storage_word(before,256+i*128)==1 and storage_word(before,264+i*128)==identity)
                assert storage_word(before,object_at+56)==0
                for slot in range(8):
                    page=1024+slot*32
                    if storage_word(before,page)==identity:
                        assert before[24+slot]==1
                        expected[24+slot]=0
                        expected[page:page+32]=bytes(32)
                        expected[4096+slot*4096:8192+slot*4096]=bytes(4096)
                expected[object_at:object_at+128]=bytes(128)
        elif n in [0,1,17,18]:
            offset=args[3] if n in [17,18] else position
            if kind==3:
                assert n==17 and result==min(args[2],256,storage_word(before_table,24)-offset)
                output=bytes.fromhex(immutable_head)[offset:offset+result]
                assert len(output)==result
            else:
                assert kind==7
                object_at=next(256+i*128 for i in range(4)
                    if storage_word(before,256+i*128)==1 and storage_word(before,264+i*128)==identity)
                size=storage_word(before,object_at+16)
                writing=n in [1,18]
                if writing:
                    assert result==min(args[2],256,4096-offset%4096)
                else:
                    assert result==min(args[2],256,4096-offset%4096,max(size-offset,0))
                if result:
                    pages=[i for i in range(8) if storage_word(before,1024+i*32)==identity
                           and storage_word(before,1032+i*32)==offset//4096]
                    if writing and not pages:
                        pages=[i for i in range(8) if storage_word(actual,1024+i*32)==identity
                               and storage_word(actual,1032+i*32)==offset//4096]
                        assert len(pages)==1
                        slot=pages[0]
                        assert before[24+slot]==0 and before[1024+slot*32:1056+slot*32]==bytes(32)
                        assert before[4096+slot*4096:8192+slot*4096]==bytes(4096)
                        expected[24+slot]=1
                        struct.pack_into('<4Q',expected,1024+slot*32,identity,offset//4096,0,0)
                    if pages:
                        assert len(pages)==1
                        start=4096+pages[0]*4096+offset%4096
                    if writing:
                        data=bytes.fromhex(call['buffer_before'])[:result]
                        assert len(data)==result
                        expected[start:start+result]=data
                        storage_put(expected,object_at+16,max(size,offset+result))
                        for timestamp in [40,48]:
                            assert storage_word(actual,object_at+timestamp)>0
                            expected[object_at+timestamp:object_at+timestamp+8]=actual[object_at+timestamp:object_at+timestamp+8]
                    else:
                        output=bytes(before[start:start+result]) if pages else bytes(result)
                    if n in [0,1]: storage_put(expected_table,desc_at+24,offset+result)
            if n in [17,18]:
                assert word(table+desc_at+24)==position
                call['positional_position_preserved']=True
        else:
            raise AssertionError(('unexpected storage call',n,result))
    assert actual==bytes(expected), (n,result,'filesystem mutation outside selected contract')
    assert memory(table,8192)==bytes(expected_table), (n,result,'description/staging mutation')
    assert pipe_context()==call['context_before']
    if 'window_before' in call:
        window=bytearray.fromhex(call['window_before'])
        if output:
            assert 1+len(output)<=len(window)
            window[1:1+len(output)]=output
            assert memory(call['buffer_pointer'],len(output))==output
            call['output_hex']=output.hex()
        assert pipe_visible(call['window_pointer'],len(window))==bytes(window), (n,result,'copy window')
        call['copy_window_verified']=True
    call['complete_storage_state_verified']=True
    call['complete_table_verified']=True

def storage_returned(row,after):
    global current
    calls=row['calls']
    assert len(calls)==52
    assert {n:sum(c['number']==n for c in calls) for n in [12,257,1,8,0,17,18,3,217]}=={
        12:3,257:3,1:5,8:5,0:7,17:15,18:5,3:3,217:6}
    actions=[c for c in calls if c['number']!=12]
    assert sum(c['result']==-14 for c in actions)==28
    assert sum(c['result']==-22 for c in actions)==1
    assert all(c['complete_storage_state_verified'] and c['complete_table_verified'] for c in actions)
    table=word(context+928)
    assert hashlib.sha256(memory(table,8192)).hexdigest()==row['storage_table_before_sha256']
    expected=bytearray.fromhex(row.pop('_storage_initial'))
    storage_put(expected,0,storage_word(expected,0)+1)
    assert memory(table+STORAGE_BASE,STORAGE_BYTES)==bytes(expected), 'filesystem reclamation/history'
    row['after']=after
    row['storage_retirement_verified']=True
    observed['executions'].append(row)
    print('SOURCE_STORAGE_CASES_PASS',len(observed['executions']),len(calls),flush=True)
    current=None
    return False



class SerialAdmission(Checked):
    def observe(self):
        assert current and pending and pending['number']==0 and pending['args'][0]==0
        assert reg('cs')&3==0 and reg('cr0')&(1<<16)
        assert reg('r8')==pending['args'][1] and reg('r9')==min(pending['args'][2],256)
        assert 'extent_call' not in pending
        pending['extent_call']=dict(candidate=reg('r8'),length=reg('r9'))
        return False

class SerialByte(Checked):
    def observe(self):
        assert current and pending and pending['number']==0 and pending['args'][0]==0
        assert reg('cs')&3==0 and reg('cr0')&(1<<16) and reg('rdx')&0xffff==0x3f8
        assert 'extent_call' in pending
        assert reg('rsi')==word(context+928)+1408+len(pending['uart_bytes'])
        pending['uart_bytes'].append(reg('rax')&255)
        assert len(pending['uart_bytes'])<=pending['extent_call']['length']
        return False

def serial_request(call):
    table=word(context+928)
    assert word(table)==1 and memory(table+1408,256)==bytes(256)
    call['table_before_sha256']=hashlib.sha256(memory(table,8192)).hexdigest()
    call['context_before']=pipe_context()
    n,args=call['number'],call['args']
    assert n in [0,1]
    call['uart_bytes']=[]
    size=min(args[2],256)
    call['buffer_before']=pipe_visible(args[1],size).hex()
    call['window_pointer']=args[1]-1
    call['window_before']=pipe_visible(args[1]-1,size+2).hex()

def serial_result(call):
    table=word(context+928)
    n,args,result=call['number'],call['args'],call['result']
    output=b''
    data=bytes(call['uart_bytes'])
    expected=b''.join(bytes.fromhex(payload) for _,payload in serial_patterns)
    if n==1:
        marker=bytes.fromhex(serial_patterns[current['serial_markers']][0])
        assert args[0]==1 and result==args[2]==len(marker)
        assert bytes.fromhex(call['buffer_before'])==marker and not data
        current['serial_markers']+=1
    elif result<0:
        assert result in [-14,-9] and not data
        assert pipe_visible(args[1],len(bytes.fromhex(call['buffer_before']))).hex()==call['buffer_before']
    elif result==0:
        assert args[0]==0 and args[2]==0 and not data and 'extent_call' not in call
    else:
        assert args[0]==0 and 0<result<=min(args[2],256) and len(data)==result
        start=len(current['serial_received'])
        assert data==expected[start:start+result]
        assert memory(args[1],result)==data
        current['serial_received']+=list(data)
        output=data
        call['output_hex']=data.hex()
    window=bytearray.fromhex(call['window_before'])
    if output:
        assert 1+len(output)<=len(window)
        window[1:1+len(output)]=output
    assert pipe_visible(call['window_pointer'],len(window))==bytes(window)
    assert hashlib.sha256(memory(table,8192)).hexdigest()==call['table_before_sha256']
    assert pipe_context()==call['context_before']
    assert call['phase_after']==call['phase_before']
    call['copy_window_verified']=True
    call['complete_table_verified']=True
    call['native_input_verified']=True

def serial_returned(row,after):
    global current
    calls=row['calls']
    assert 16<=len(calls)<=29
    assert sum(c['number']==12 for c in calls)==3 and sum(c['number']==1 for c in calls)==2
    assert all(c['number'] in [0,1,12] for c in calls)
    assert sum(c['result']==-14 for c in calls)==5
    assert sum(c['result']==-9 for c in calls)==2
    assert row['serial_markers']==2
    assert bytes(row['serial_received'])==b'Uabcdefg12345678'
    requests=[c for c in calls if c['number']!=12]
    assert all(c['copy_window_verified'] and c['complete_table_verified'] and c['native_input_verified'] for c in requests)
    large=[c for c in requests if c['number']==0 and c['args'][2]==512]
    assert len(large)==1 and large[0]['extent_call']==dict(candidate=0x639f00,length=256)
    assert hashlib.sha256(memory(word(context+928),8192)).hexdigest()==row['serial_table_before_sha256']
    row['after']=after
    row['serial_input_verified']=True
    observed['executions'].append(row)
    print('SOURCE_SERIAL_CASES_PASS',len(observed['executions']),len(calls),flush=True)
    current=None
    return False

def install_serial_observation():
    if component.get('mode')=='serial':
        SerialAdmission('*'+hex(serial_sites['admission']),internal=True)
        SerialByte('*'+hex(serial_sites['byte_received']),internal=True)


class Complete(Checked):
    def observe(self):
        assert current is startup_current is pending is bridge_current is None and observed['executions']
        numbers = [row['number'] for row in observed['startup_executions']]
        if component.get('mode') in ['exec','signals','poll','pipes','storage','serial']:
            assert len(observed['executions']) == 4
            assert all(not observed[key] for key in [
                'startup_executions','file_executions','shell_executions','command_executions'])
        else:
            assert numbers and len(numbers) % 5 == 0
        assert numbers == [302,318,318,157,262]*(len(numbers)//5)
        assert [row['number'] for row in observed['file_executions']] == ([] if component.get('mode') in ['exec','signals','poll','pipes','storage','serial'] else [
            5,257,262,257,5,0,257,5,217,217,262,257,5,217,217,262,262,262,257,257]
        )
        assert len(observed['command_executions']) == (0 if component.get('mode') in ['exec','signals','poll','pipes','storage','serial'] else 5)
        assert len(observed['shell_executions']) == (0 if component.get('mode') in ['exec','signals','poll','pipes','storage','serial'] else 38)
        assert len(observed['poll_executions']) == (4 if component.get('mode') == 'poll' else 0)
        assert len(observed['bridge_returns']) == len(numbers)+len(observed['file_executions'])+len(observed['shell_executions'])+len(observed['command_executions'])+len(observed['poll_executions'])
        assert state() == [0,0,0,0]
        assert memory(address('topal_x86_frame_mm_tokens_v1'), 48) == bytes(48)
        assert word(context+904) == 0 and memory(context+152,16) == bytes(16)
        assert memory(context+416,8) == bytes(8)
        for snapshot in {row['snapshot'] for row in observed['executions']}:
            assert memory(snapshot+0x9e00,512) == bytes(512)
        extent_coverage_complete()
        observed['passed'] = True
        with open(observation_path, 'w') as handle:
            json.dump(observed, handle, indent=2)
        return True

class InvariantHalt(Checked):
    def observe(self):
        observed['invariant_failure'] = dict(phase=memory(context+104,1)[0],
            heap=state(),current=current,pending=pending)
        with open(observation_path, 'w') as handle:
            json.dump(observed, handle, indent=2)
        print('SOURCE_USER_INVARIANT_FAILURE', observed['invariant_failure'], flush=True)
        return True

try:
    install_extent_coverage()
    install_serial_observation()
    SourceEntered('*'+hex(component['entry']), internal=True)
    StartupEntered('*'+hex(component['startup']), internal=True)
    BridgeEntered('*'+hex(component['startup_call']), internal=True)
    SyscallEntered('*'+hex(component['syscall']), 3)
    SyscallEntered('*'+hex(component['syscall4']), 4)
    InvariantHalt('*'+hex(native_sites['halt']), internal=True)
    Complete('*topal_x86_systems_fatal', internal=True)
    gdb.execute('continue')
    assert observed['passed']
    print('FRAME_ROOT_GDB_PASS')
    gdb.execute('detach')
except Exception:
    traceback.print_exc()
    gdb.execute('quit 1')

end
