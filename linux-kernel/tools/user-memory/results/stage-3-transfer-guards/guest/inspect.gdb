set pagination off
set confirm off
set remotetimeout 10
file "/home/mats/git/topal/linux-kernel/.cache/x86_64/user-extent-application-2276559-1794401643985043835/kernel.elf"
target remote 127.0.0.1:60537
python
token_bytes=336
observation_path='/home/mats/git/topal/linux-kernel/.cache/x86_64/user-extent-application-2276559-1794401643985043835/first-image-exit/observations.json'
native_sites={'halt': 45795222, 'exits': [45689936, 45691580]}
extent_inventory={'function': 'topal_x86_systems_static_image_process_enter', 'address': 45671296, 'calls': [{'nearest_labels': ['static-process-shell-uname'], 'offset': 27139, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-getcwd'], 'offset': 27861, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-chdir-root-profile-ready'], 'offset': 28626, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-pwd-getcwd'], 'offset': 29345, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-initial-getcwd'], 'offset': 29969, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['owned-header-poll-profile-ready'], 'offset': 33496, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-header-poll-ready'], 'offset': 33583, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-query-default-common'], 'offset': 36023, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-install-quit-ignore'], 'offset': 36684, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-read-fstat-ready'], 'offset': 38450, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-refill-command-read-facts-ready'], 'offset': 39534, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-root-test-command-read-profile-ready'], 'offset': 40444, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-header-read-profile-ready'], 'offset': 41322, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-pwd-command-read-profile-ready'], 'offset': 42436, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['owned-shell-serial-copy'], 'offset': 43425, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['runtime-exec-path-byte'], 'offset': 44431, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 166, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['runtime-exec-vector-entry'], 'offset': 44792, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 189, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['runtime-exec-argument-byte'], 'offset': 44888, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 206, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-console-fstat-candidate-ready'], 'offset': 52962, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-dev-tty-open-miss-exact'], 'offset': 55270, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-profile-open'], 'offset': 56053, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-open-clean', 'static-process-shell-bin-busybox-open-profile-ready'], 'offset': 56936, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-reopen-profile-ready'], 'offset': 57843, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-root-glob-open-profile-ready'], 'offset': 58659, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-empty-bin-glob-open-profile-ready'], 'offset': 59435, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-root-glob-fstat-profile-ready'], 'offset': 60174, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-empty-bin-glob-fstat-profile-ready'], 'offset': 62337, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-profile-fstat'], 'offset': 64531, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-root-glob-getdents-first'], 'offset': 68177, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-empty-bin-glob-getdents-first'], 'offset': 70022, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-applet-newline-candidate-ready'], 'offset': 75142, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-explicit-close-write-clean', 'static-process-shell-bin-busybox-explicit-close-write-profile-ready'], 'offset': 76007, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-warning-candidate'], 'offset': 76915, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-prompt'], 'offset': 80006, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-refill-prompt'], 'offset': 81477, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-root-condition-write-clean', 'static-process-shell-root-condition-write-profile-ready'], 'offset': 82375, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-empty-bin-glob-write-clean', 'static-process-shell-empty-bin-glob-write-profile-ready'], 'offset': 83237, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-executable-write-clean', 'static-process-shell-bin-busybox-executable-write-profile-ready'], 'offset': 84283, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-nonempty-write-clean', 'static-process-shell-bin-busybox-nonempty-write-profile-ready'], 'offset': 85284, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-open-write-clean', 'static-process-shell-bin-busybox-open-write-profile-ready'], 'offset': 86299, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-header-write-profile-ready'], 'offset': 87305, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-named-root-condition-write-clean', 'static-process-shell-named-root-condition-write-profile-ready'], 'offset': 88236, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-pwd-write-profile-ready'], 'offset': 89191, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-third-prompt'], 'offset': 90027, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-fourth-prompt-profile-ready'], 'offset': 91024, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-banner'], 'offset': 92125, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-echo-candidate-ready'], 'offset': 93521, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-execve-signal-number-ready'], 'offset': 94708, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-sigchld-install'], 'offset': 95395, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-initial-process-name'], 'offset': 95961, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-bin-busybox-stat-path'], 'offset': 98915, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-named-root-directory-stat-path'], 'offset': 99039, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-named-root-directory-stat-path-ready'], 'offset': 99332, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-shell-root-directory-stat-profile-ready'], 'offset': 101802, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-shell-mailpath-stat-miss'], 'offset': 102873, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-clean-config-path', 'static-process-config-path-ready'], 'offset': 103476, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['static-process-stack-limit'], 'offset': 107362, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['static-process-getrandom'], 'offset': 108204, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 365, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['runtime-console-payload', 'runtime-write-fd-ready'], 'offset': 131616, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_runtime.rs', 'source_line': 427, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['fd-pread-count-ready'], 'offset': 140542, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_positional_transfer.rs', 'source_line': 60, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['fd-read-count-ready'], 'offset': 141402, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_file_descriptions.rs', 'source_line': 480, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['fd-serial-count-ready'], 'offset': 142334, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_file_descriptions.rs', 'source_line': 514, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['fd-stat'], 'offset': 143474, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_file_descriptions.rs', 'source_line': 586, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['fd-limit-publish'], 'offset': 144285, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_file_descriptions.rs', 'source_line': 649, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['process-signal-action'], 'offset': 145214, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_signals.rs', 'source_line': 39, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['process-signal-action-publish'], 'offset': 145584, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_signals.rs', 'source_line': 53, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['process-signal-mask'], 'offset': 145783, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_signals.rs', 'source_line': 39, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['process-signal-mask-publish'], 'offset': 145968, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_signals.rs', 'source_line': 53, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['namespace-copy-byte'], 'offset': 151753, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_namespace_walk.rs', 'source_line': 151, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['namespace-stat-object'], 'offset': 155270, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_namespace.rs', 'source_line': 327, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['namespace-cwd-length'], 'offset': 156830, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_namespace.rs', 'source_line': 422, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['namespace-directory-copy'], 'offset': 158576, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_namespace.rs', 'source_line': 607, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['named-directory-publish'], 'offset': 161434, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_namespace_named.rs', 'source_line': 360, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['realtime-seconds'], 'offset': 177579, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_realtime.rs', 'source_line': 45, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['process-observations-uname'], 'offset': 185975, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_observations.rs', 'source_line': 60, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['process-time-gettime'], 'offset': 186810, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_time.rs', 'source_line': 76, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['process-time-getres'], 'offset': 186999, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_time.rs', 'source_line': 76, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['process-time-copy-interval'], 'offset': 187213, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_time.rs', 'source_line': 139, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['child-thread-local'], 'offset': 190244, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_children.rs', 'source_line': 136, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['child-clear-registration-ready'], 'offset': 194631, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_children.rs', 'source_line': 316, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['child-wait-matched'], 'offset': 196557, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_children.rs', 'source_line': 413, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['pipe-descriptions-ready'], 'offset': 203337, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_pipe_create.rs', 'source_line': 109, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['pipe-read-count-ready'], 'offset': 204492, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_pipe_transfer.rs', 'source_line': 56, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['pipe-write-count-ready'], 'offset': 205322, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_pipe_transfer.rs', 'source_line': 131, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['pipe-ioctl'], 'offset': 207832, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_process_pipes.rs', 'source_line': 152, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['file-read-end'], 'offset': 221407, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_volatile_transfer.rs', 'source_line': 59, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['file-pread-end'], 'offset': 222189, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_volatile_transfer.rs', 'source_line': 59, 'target': 'runtime-writable-extent'}, {'nearest_labels': ['file-write-end'], 'offset': 222985, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_volatile_transfer.rs', 'source_line': 201, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['file-pwrite-end'], 'offset': 224056, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_volatile_transfer.rs', 'source_line': 201, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['runtime-environment-entry'], 'offset': 231462, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_exec_environment.rs', 'source_line': 25, 'target': 'runtime-readable-extent'}, {'nearest_labels': ['runtime-environment-byte'], 'offset': 231558, 'source_column': 9, 'source_file': 'src/topal-compiler/src/systems_exec_environment.rs', 'source_line': 41, 'target': 'runtime-readable-extent'}], 'kernel_sha256': '04f4f0b01552e5d724187ae841bec2844d00d113eee2ee940adb8304997433c6', 'inventory_sha256': '1408d6f9916bd5a0c37000b98ce1e8c38e6bad793e780366177b9af4e9136711'}


import gdb
import json
import struct
import traceback

def reg(name): return int(gdb.parse_and_eval('$'+name)) & ((1<<64)-1)
def address(name): return int(gdb.parse_and_eval('(unsigned long long)&'+name))
def memory(pointer, size): return bytes(gdb.selected_inferior().read_memory(pointer, size))
def word(pointer): return struct.unpack('<Q', memory(pointer, 8))[0]
def words(pointer, count): return list(struct.unpack('<'+'Q'*count, memory(pointer, count*8)))

context = address('topal_bootstrap_storage') + 512
epoch = address('topal_x86_frame_mm_epoch_v1')
observed = dict(passed=False)

class CheckedBreakpoint(gdb.Breakpoint):
    def stop(self):
        try:
            return self.observe()
        except Exception:
            traceback.print_exc()
            observed['observer_failure'] = traceback.format_exc()
            with open(observation_path, 'w') as handle:
                json.dump(observed, handle, indent=2)
            return True

Checked = CheckedBreakpoint


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


def consumed_prelude():
    assert all(word(context + offset) == 0 for offset in [168, 176, 224, 232])
    assert memory(context + 417, 1) == bytes(1)

class SerialReturned(CheckedBreakpoint):
    def observe(self):
        self.enabled = False
        assert reg('cs') & 3 == 3 and reg('rax') == 5
        assert memory(context + 104, 1) == bytes([32])
        assert memory(context + 418, 1) == bytes([6])
        snapshot = observed['snapshot']
        assert word(snapshot + 0x9f10) == 5 and word(snapshot + 0x9f08) == 0
        consumed_prelude()
        table = observed['table']
        assert word(table) == 1
        child = words(table + 1792 + 16*32, 4)
        quit_action = words(table + 1792 + 2*32, 4)
        expected = observed['prelude_child'][:]
        expected[3] &= ~0x40100
        assert child == expected, (child, expected)
        assert quit_action == [1, 0x4000000, 0x4a9740, expected[3]], quit_action
        observed.update(imported_child=child, imported_quit=quit_action)
        observed['serial_delivered'] = True
        return False

class SerialInput(CheckedBreakpoint):
    def observe(self):
        if (word(epoch + 8) != 0 or memory(context + 104, 1) != bytes([31])
                or word(context + 312) != 0):
            return False
        table = word(context + 928)
        observed['serial_admission'] = dict(owner=memory(context + 418, 1)[0],
            count=reg('r9'), child=words(table + 1792 + 16*32, 4),
            quit_action=words(table + 1792 + 2*32, 4))
        assert 'table' not in observed and memory(context + 418, 1) == bytes([5])
        assert reg('cs') & 3 == 0 and reg('cr0') & (1 << 16) and reg('r9') == 5
        table = word(context + 928)
        assert table and memory(table, 8192) == bytes(8192)
        assert memory(context + 417, 1) == bytes([1])
        child = [word(context + offset) for offset in [168, 176, 224, 232]]
        assert 0x401000 <= child[0] < 0x58c285 and child[1] == 0x4000000
        assert child[2] == 0x4a9740
        assert child[3] == 0xfffffffe7fffffff
        observed.update(table=table, snapshot=word(context + 904),
                        prelude_child=child)
        SerialReturned('*'+hex(word(context + 296)), internal=True)
        return False

class ExitSelected(CheckedBreakpoint):
    def observe(self):
        assert observed['serial_delivered'] and word(epoch + 8) == 231
        assert word(context + 312) == 0 and word(context + 928) == observed['table']
        assert word(observed['table']) == 1
        consumed_prelude()
        observed['terminal_child'] = words(observed['table'] + 1792 + 16*32, 4)
        observed['terminal_quit'] = words(observed['table'] + 1792 + 2*32, 4)
        observed['exit_selected'] = True
        return False

class Complete(CheckedBreakpoint):
    def observe(self):
        assert observed['exit_selected']
        assert memory(observed['snapshot'] + 0x9e00, 512) == bytes(512)
        assert memory(observed['table'], 8192) == bytes(8192)
        assert word(context + 904) == 0
        assert memory(context + 152, 16) == bytes(16)
        assert memory(context + 416, 8) == bytes(8)
        assert memory(address('topal_x86_frame_mm_state_v1'), 32) == bytes(32)
        assert memory(address('topal_x86_frame_mm_tokens_v1'), 48) == bytes(48)
        extent_coverage_complete()
        observed['passed'] = True
        with open(observation_path, 'w') as handle:
            json.dump(observed, handle, indent=2)
        return True

class InvariantHalt(CheckedBreakpoint):
    def observe(self):
        observed['invariant_failure'] = dict(
            syscall=word(epoch + 8), phase=memory(context + 104, 1)[0],
            owner=memory(context + 418, 1)[0],
            signal_state=[word(context + offset) for offset in [168, 176, 224, 232]],
        )
        print('OWNED_EXIT_INVARIANT_FAILURE', observed['invariant_failure'], flush=True)
        with open(observation_path, 'w') as handle:
            json.dump(observed, handle, indent=2)
        return True

try:
    install_extent_coverage()
    SerialInput('*topal_x86_user_extent_writable_v1', internal=True)
    Complete('*topal_x86_systems_fatal', internal=True)
    InvariantHalt('*'+hex(native_sites['halt']), internal=True)
    for site in native_sites['exits']:
        ExitSelected('*'+hex(site), internal=True)
    gdb.execute('continue')
    assert observed['passed']
    print('FRAME_ROOT_GDB_PASS')
    gdb.execute('detach')
except Exception:
    traceback.print_exc()
    gdb.execute('quit 1')

end
