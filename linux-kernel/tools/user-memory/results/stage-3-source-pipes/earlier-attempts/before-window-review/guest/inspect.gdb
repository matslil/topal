set pagination off
set confirm off
set remotetimeout 10
file "/home/mats/git/topal/linux-kernel/.cache/x86_64/user-extent-source-pipes-2270539-1794398764192615193/kernel.elf"
target remote 127.0.0.1:50103
python
token_bytes=336
observation_path='/home/mats/git/topal/linux-kernel/.cache/x86_64/user-extent-source-pipes-2270539-1794398764192615193/source-user-extents/observations.json'
component={'base': 5701632, 'command_source_sha256': 'cac5afad81808c0123175a2de8b1e799c70579f9a5ef81445bb3a66433f9e3ef', 'conservative_user_stack_bytes': 1352, 'entry': 5708672, 'exec_source_sha256': None, 'file_source_sha256': '757a45600b38e0746659781ba6e14aa5a90ec9d378cf364fd555299890ef92fa', 'image_sha256': '1fe291d3850131cea0d374be8ec9c0225cf953e5f4923b502b65453492338b12', 'linked_sha256': '9129707fa6d352e236cf5cc14025f1bde0cea54b3677066cc4ceedb9593604c3', 'mode': 'pipes', 'pipe_source_sha256': '9c60c23c96c582163f8688c7e5fcb2253808cc9da8fa535218a8d98a92beec67', 'poll_source_sha256': None, 'scope': 'source-built ordinary syscall component in a freshly generated first-party ABI witness; not an arbitrary-program loader or production qualification', 'shell_source_sha256': '574462c620fb4e2db01bc4140e98377f52d4add8eddc3e4a9fa0428d2a3d8db4', 'signal_source_sha256': None, 'source_sha256': '4350abed34fe56b3961d52ec3d73532a03de56320cf08f84d0cac5688538d70b', 'stack_frame_costs': {'command-cases': 104, 'file-cases': 400, 'pipe-cases': 200, 'shell-cases': 168, 'source-cases': 496, 'startup-cases': 168}, 'startup': 5702592, 'startup_call': 5710496, 'startup_source_sha256': '2c5171f997c27d084883f6dce969747c191ab41649864e5c1d73f2367bb574aa', 'syscall': 5710448, 'syscall4': 5710464, 'text_bytes': 9009}
extent_inventory={'function': 'topal_x86_systems_static_image_process_enter', 'address': 45671296, 'calls': [{'nearest_labels': ['static-process-shell-uname'], 'offset': 27139, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-getcwd'], 'offset': 27861, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-chdir-root-profile-ready'], 'offset': 28626, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-pwd-getcwd'], 'offset': 29345, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-initial-getcwd'], 'offset': 29969, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['owned-header-poll-profile-ready'], 'offset': 33496, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-header-poll-ready'], 'offset': 33583, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-query-default-common'], 'offset': 36023, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-install-quit-ignore'], 'offset': 36684, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-read-fstat-ready'], 'offset': 38450, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-refill-command-read-facts-ready'], 'offset': 39534, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-root-test-command-read-profile-ready'], 'offset': 40444, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-header-read-profile-ready'], 'offset': 41322, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-pwd-command-read-profile-ready'], 'offset': 42436, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['owned-shell-serial-copy'], 'offset': 43425, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['runtime-exec-path-byte'], 'offset': 44431, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 166, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['runtime-exec-vector-entry'], 'offset': 44792, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 189, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['runtime-exec-argument-byte'], 'offset': 44888, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 206, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-console-fstat-candidate-ready'], 'offset': 52962, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-dev-tty-open-miss-exact'], 'offset': 55270, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-profile-open'], 'offset': 56053, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-open-clean', 'static-process-shell-bin-busybox-open-profile-ready'], 'offset': 56936, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-reopen-profile-ready'], 'offset': 57843, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-root-glob-open-profile-ready'], 'offset': 58659, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-empty-bin-glob-open-profile-ready'], 'offset': 59435, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-root-glob-fstat-profile-ready'], 'offset': 60174, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-empty-bin-glob-fstat-profile-ready'], 'offset': 62337, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-profile-fstat'], 'offset': 64531, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-root-glob-getdents-first'], 'offset': 68177, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-empty-bin-glob-getdents-first'], 'offset': 70022, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-applet-newline-candidate-ready'], 'offset': 75142, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-explicit-close-write-clean', 'static-process-shell-bin-busybox-explicit-close-write-profile-ready'], 'offset': 76007, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-warning-candidate'], 'offset': 76915, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-prompt'], 'offset': 80006, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-refill-prompt'], 'offset': 81477, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-root-condition-write-clean', 'static-process-shell-root-condition-write-profile-ready'], 'offset': 82375, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-empty-bin-glob-write-clean', 'static-process-shell-empty-bin-glob-write-profile-ready'], 'offset': 83237, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-executable-write-clean', 'static-process-shell-bin-busybox-executable-write-profile-ready'], 'offset': 84283, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-nonempty-write-clean', 'static-process-shell-bin-busybox-nonempty-write-profile-ready'], 'offset': 85284, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-open-write-clean', 'static-process-shell-bin-busybox-open-write-profile-ready'], 'offset': 86299, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-header-write-profile-ready'], 'offset': 87305, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-named-root-condition-write-clean', 'static-process-shell-named-root-condition-write-profile-ready'], 'offset': 88236, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-pwd-write-profile-ready'], 'offset': 89191, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-third-prompt'], 'offset': 90027, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-fourth-prompt-profile-ready'], 'offset': 91024, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-banner'], 'offset': 92125, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-echo-candidate-ready'], 'offset': 93521, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-execve-signal-number-ready'], 'offset': 94708, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-sigchld-install'], 'offset': 95395, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-initial-process-name'], 'offset': 95961, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-stat-path'], 'offset': 98915, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-named-root-directory-stat-path'], 'offset': 99039, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-named-root-directory-stat-path-ready'], 'offset': 99332, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-root-directory-stat-profile-ready'], 'offset': 101802, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-mailpath-stat-miss'], 'offset': 102873, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-clean-config-path', 'static-process-config-path-ready'], 'offset': 103476, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-stack-limit'], 'offset': 107362, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-getrandom'], 'offset': 108204, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['runtime-console-payload', 'runtime-write-fd-ready'], 'offset': 131616, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 427, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['fd-pread-count-ready'], 'offset': 140542, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_positional_transfer.rs', 'source_line': 60, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['fd-read-count-ready'], 'offset': 141402, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_file_descriptions.rs', 'source_line': 480, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['fd-serial-count-ready'], 'offset': 142334, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_file_descriptions.rs', 'source_line': 514, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['fd-stat'], 'offset': 143474, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_file_descriptions.rs', 'source_line': 586, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['fd-limit-publish'], 'offset': 144285, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_file_descriptions.rs', 'source_line': 649, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['process-signal-action'], 'offset': 145214, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_signals.rs', 'source_line': 39, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['process-signal-action-publish'], 'offset': 145584, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_signals.rs', 'source_line': 53, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['process-signal-mask'], 'offset': 145783, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_signals.rs', 'source_line': 39, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['process-signal-mask-publish'], 'offset': 145968, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_signals.rs', 'source_line': 53, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['namespace-copy-byte'], 'offset': 151753, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_namespace_walk.rs', 'source_line': 151, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['namespace-stat-object'], 'offset': 155270, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_namespace.rs', 'source_line': 327, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['namespace-cwd-length'], 'offset': 156830, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_namespace.rs', 'source_line': 422, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['namespace-directory-copy'], 'offset': 158576, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_namespace.rs', 'source_line': 607, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['named-directory-publish'], 'offset': 161434, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_namespace_named.rs', 'source_line': 360, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['realtime-seconds'], 'offset': 177579, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_realtime.rs', 'source_line': 45, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['process-observations-uname'], 'offset': 185975, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_observations.rs', 'source_line': 60, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['process-time-gettime'], 'offset': 186810, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_time.rs', 'source_line': 76, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['process-time-getres'], 'offset': 186999, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_time.rs', 'source_line': 76, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['process-time-copy-interval'], 'offset': 187213, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_time.rs', 'source_line': 139, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['child-thread-local'], 'offset': 190244, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_children.rs', 'source_line': 136, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['child-clear-registration-ready'], 'offset': 194631, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_children.rs', 'source_line': 316, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['child-wait-matched'], 'offset': 196557, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_children.rs', 'source_line': 413, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['pipe-descriptions-ready'], 'offset': 203337, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_pipe_create.rs', 'source_line': 109, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['pipe-read-count-ready'], 'offset': 204492, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_pipe_transfer.rs', 'source_line': 56, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['pipe-write-count-ready'], 'offset': 205322, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_pipe_transfer.rs', 'source_line': 131, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['pipe-ioctl'], 'offset': 207832, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_pipes.rs', 'source_line': 152, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['file-read-end'], 'offset': 221407, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_volatile_transfer.rs', 'source_line': 59, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['file-pread-end'], 'offset': 222189, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_volatile_transfer.rs', 'source_line': 59, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['file-write-end'], 'offset': 222985, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_volatile_transfer.rs', 'source_line': 201, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['file-pwrite-end'], 'offset': 224056, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_volatile_transfer.rs', 'source_line': 201, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['runtime-environment-entry'], 'offset': 231462, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_exec_environment.rs', 'source_line': 25, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['runtime-environment-byte'], 'offset': 231558, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_exec_environment.rs', 'source_line': 41, 'target': 'runtime-readable-extent'}], 'kernel_sha256': '04f4f0b01552e5d724187ae841bec2844d00d113eee2ee940adb8304997433c6', 'inventory_sha256': '1408d6f9916bd5a0c37000b98ce1e8c38e6bad793e780366177b9af4e9136711'}
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
        assert address(extent_inventory['function']) == extent_inventory['address']
        for row in extent_inventory['calls']:
            ExtentFirstReach(row)



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
            struct.pack_into('<QQ',expected_table,64+fd*16,description,0)
            struct.pack_into('<8Q',expected_table,320+(description-1)*64,
                             5+endpoint,1,0x800|endpoint,0,generation,0,0,0)
        record=list(struct.unpack_from('<16Q',after,index*128))
        assert record[:9]==[1,generation,*descs,0,0,0,0,0o10600]
        assert record[9]==record[10]==record[11]==record[12]>0 and record[13:]==[0]*3
        expected[index*128:index*128+128]=after[index*128:index*128+128]
        struct.pack_into('<Q',expected,512,generation)
    else:
        index=current['pipe_index']
        at=index*128
        ring=0x1000+index*4096
        cursor,count=struct.unpack_from('<QQ',before,at+32)
        if n==16:
            assert result==0 and args[1]==0x541b
            assert memory(args[2],4)==struct.pack('<I',count)
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


class Complete(Checked):
    def observe(self):
        assert current is startup_current is pending is bridge_current is None and observed['executions']
        numbers = [row['number'] for row in observed['startup_executions']]
        if component.get('mode') in ['exec','signals','poll','pipes']:
            assert len(observed['executions']) == 4
            assert all(not observed[key] for key in [
                'startup_executions','file_executions','shell_executions','command_executions'])
        else:
            assert numbers and len(numbers) % 5 == 0
        assert numbers == [302,318,318,157,262]*(len(numbers)//5)
        assert [row['number'] for row in observed['file_executions']] == ([] if component.get('mode') in ['exec','signals','poll','pipes'] else [
            5,257,262,257,5,0,257,5,217,217,262,257,5,217,217,262,262,262,257,257]
        )
        assert len(observed['command_executions']) == (0 if component.get('mode') in ['exec','signals','poll','pipes'] else 5)
        assert len(observed['shell_executions']) == (0 if component.get('mode') in ['exec','signals','poll','pipes'] else 38)
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
