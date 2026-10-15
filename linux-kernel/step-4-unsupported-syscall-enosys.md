# Step 4 unsupported-syscall `ENOSYS` fallback

## Outcome

The initial process now returns Linux-compatible `-ENOSYS` for every syscall
number not selected by an implemented dispatch arm. The fallback returns
through the ordinary executable syscall continuation without inspecting unused
arguments, accessing user memory, changing process state, or terminating the
process.

The constructed application invokes deliberately unsupported syscall number
`0x7fffffff`, validates the exact negative errno, and then continues into its
existing successful stdout operation. Later syscall increments replace this
fallback one dispatch arm at a time.

## Validation boundary

Rule `TOPAL-SYSTEMS-USER-ENOSYS-001` requires exact native `-ENOSYS`, no argument
or state effects, a nonterminal return, and continued implemented dispatch.
Structural tests prove the fallback return sequence and the constructed probe's
single unsupported invocation.

Artifact schema `/69`, provider `/66`, provider object `/66`, artifact root
`/67`, boot adapter `/29`, and gate `/71` identify the changed contract. The
boot-resource schema remains `/1`. Focused language, semantics, compiler, and
gate-fixture tests pass, as does the complete workspace suite.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 51 ordered observations. The evidence records:

- linked kernel: `9e2ea38e31cecb1cc6ecc6e304af5be8efac10b4b04782a12dcf365a2d764e1a`;
- boot image: `4956a66d0b3c8a4c6004680c7af37f3cbe802f6b5ff0a77aed6aee3931c98543`;
- boot resource: `bb6eeadd807464237bbd2f53dbce5b34354344c1c6ad8e6ee4bdbd000d77e1a8`;
- generated default image: `e36b8ced391f9c29cef20ce84027636ad90ae4de833f188b025b45962a404bd6`;
- supplied image: `6cbbd8320e366ec58741458a36284f93255639c893d79250efbedfbc14daa4c5`;
- serial input: `3316348dbadfb7b11c7c2ea235949419e23f9fa898ad2c198f999617912a9925`;
- artifact provenance: `d4096d6690f82302b984a29b07cba6c6a3e96d11d7da86ace3d75d233095cdac`;
- boot provenance: `e3083cdb8d04de8cfc0c76d919da28873fc80fe59a5f805851deddb8b148ffe9`;
- serial output: `af426f6546f91c1f48d2b81d20c06bbf5de67aa7794bf51553738454320043ba`;
- evidence record: `e3671c3134c960d265370f63dbd06261ad5835c6127e9e527151930217a478b1`.

## Risk and review

Risk is medium: a dispatch default affects every syscall not yet implemented,
but the correct behavior is side-effect-free and narrow. Review covered arm
ordering, the exact signed x86-64 result, executable return validation, absence
of argument access and state changes, continued execution, structural
inspection, full workspace validation, and two physical runs.

## Remaining boundary

The fallback does not implement any missing syscall and does not add seccomp,
audit, tracing, syscall introspection, signal generation, or compatibility
ABIs. It only makes unsupported-operation discovery Linux-compatible.
