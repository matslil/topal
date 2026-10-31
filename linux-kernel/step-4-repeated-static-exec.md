# Step 4 increment: successive static image replacements

A process can now replace its image more than once while retaining the same
admitted immutable executable. A later generation validates its own completed
startup instead of requiring the consumed original shell-command phase.
Unmodified BusyBox `env` can exec another BusyBox image repeatedly and finish
in `echo`, `pwd` or `false`.

This is a Step 4 increment under systems decision `TK-DEC-246` and rule
`TOPAL-SYSTEMS-STATIC-PROCESS-EXEC-REPEAT-001`. It changes no core-language
meaning. Artifact `/222`, root `/211`, provider `/212`, provider object `/204`,
boot `/122` and gate `/222` identify the implementation. Distribution boot,
Docker/Podman and VM hosting remain unfinished.

## Generation and process lifetimes

The provider retains one immutable executable, pristine writable snapshot,
process descriptor table, cwd, identity and signal state for the process
lifetime. The initial handover still verifies the predecessor command's
qualification facts. Subsequent replacements use the retained replacement
owner and current completed-startup state. Every request also checks the live
address space and exact backing. No replacement counter or applet identity
decides permission.

Each request copies all bounded argv/envp bytes before destructive commit.
Failures erase staging and return to the current image. Success filters the
current close-on-exec slots, resets caught handlers and action metadata,
preserves ignored handlers/mask and eligible descriptions/positions, restores
pristine image bytes, clears BSS/old anonymous bytes, revokes anonymous mappings
and reconstructs the stack. Each entry extracts fresh random bytes from the
retained generator, resets TLS/register/SIMD state and uses its actual vector
and auxiliary identities. Snapshot and process resources remain owned until
the single final terminal-release/outer-reclamation path.

The exact complete 34-page anonymous-backing qualification remains a
precondition. This change does not establish sparse/arbitrary mapping layouts,
general exec-path selection, another executable, a dynamic interpreter or
multiple processes.

## Executed evidence

The maximum clean ABI fixture performs three successful replacements. A
caller-owned nonzero environment byte carries its own progress across images;
the kernel neither interprets that byte nor controls the fixture loop. Each
generation checks vectors and transported bytes, uses the stack below entry,
observes the same file offset and cwd, checks ignored/cleared signal handlers
and retained mask, and retries after a copied-argument-prefix failure. Before
each additional replacement it creates a fresh close-on-exec alias and caught
handler; the next generation verifies both were reset. After the third entry,
the existing file/directory/signal/error corpus completes and exits with status
`0x23`. Empty and null environment variants remain one-replacement regressions.
Rust and Python independently encode the default executed fixture.

Three external cases extend the standard corpus:

| Case | Successful replacements | Output | Status |
| --- | --- | --- | --- |
| `exec-chain` | 3 | `chain\n` | 0 |
| `exec-chain-false` | 2 | empty | 1 |
| `exec-chain-cwd` | 3 | `/bin\n` | 0 |

The kernel emits one private commit marker immediately before each replacement
entry. Evidence records the actual marker count; failed requests emit none.
The harness checks counts independently of application output, and removes
only those declared markers when comparing the exact application transcript.
The complete raw serial transcript and digest remain in evidence. There is one
final status observation and one ordered outer rollback, with QEMU still alive
and serial quiescent.

The qualification matrix contains three clean variants, eighteen standard
external cases and five RTC cases. All use linked kernel SHA-256
`e67026b14653b540d39a28e9fbc064b68051d42017c3c1296230ab0239edda9a`.
The full bounded Cargo workspace, including 196 compiler tests, eleven lab
contract tests, formatting, whitespace, document links and the 234-rule
inventory all pass.

## Risk review and remaining work

Risk is high because destructive image transitions repeat under one owner.
Review follows the initial/later admission split, live backing checks, copied
inputs and error paths, repeated filtering/reset, snapshot ownership, generator
extraction, current stack/auxiliary construction, private-stack use and final
release. The existing executed boundary corpus, independently encoded clean
fixture and unmodified external exec chains supplement self-review; human PR
review and merge remain required.

The [design](design/repeated-static-exec.md) keeps portable image-generation
lifetime separate from target reset/entry mechanisms. General executable
selection and mapping, null/empty argv normalization, dynamic ELF, process
creation/waiting, writable filesystems, networking, distribution boot,
containers and VM-host compatibility remain required work in the initial plan.
