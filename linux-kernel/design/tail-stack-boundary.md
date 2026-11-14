# Private source-stack boundary

This refines `TK-DEC-282` Stage 2 and the private step component. Source still
uses ordinary functions and region operations; target stack/register details
belong to the systems provider, not source assembly or numeric authority.

The x86-64 component receives an owning caller's mapped, exclusive temporary
extent and disjoint 4096-byte source stack. An eight-byte instance recovery
cell plus 65528 caller-supplied temporary bytes retain the complete 65536-byte
temporary limit. The first 64 supplied bytes are private recovery metadata;
the opaque step context follows. Caller input, persistent history, owner bytes,
code/data and live caller frames remain disjoint and outside erasure.

The provider binds the step entry and nonreturning fatal entry by exact native
symbol/identity. Pointer/alignment/end/nonoverlap tests do not create ownership
or mapped-storage evidence. A busy instance or malformed pre-entry geometry
returns a private refusal without changing the active session or histories.
The owning kernel caller must also establish the interrupt/context conditions
required by its qualified provider; user-mode component tests do not prove them.

After preserving the caller's callee-saved state, the provider pins recovery
information, enters the private stack and calls the step. An ordinary return
makes all source/helper frames dead. It restores the caller stack, erases and
reads back the complete source stack, clears the recovery header/cell and
volatile registers, then returns only the private outcome. State banks remain
live until the controller completes transfer/reclamation. Invalid outcomes
enter fatal recovery rather than resuming the owner.

An admitted abort finds the pinned recovery record independently of the
source's current registers. It returns to the saved caller stack, erases the
entire temporary extent and source stack, clears instance bindings/registers,
and enters the bound nonreturning fatal handler. Already-performed source
stores and persistent history are retained. A test observer may inspect that
disposition; its observer escape is not a kernel recovery or owner resumption.

Accurate CFI must describe saved caller registers and the stack pivot. Complete
native code/data/hook identity, scalar-only provider/runtime code, resource
bounds, scope/session model and independent guarded/GDB evidence remain
qualification obligations. This component alone grants no kernel publication.
AArch64/RISC-V need their own calling-convention, stack and ordering providers.

The x86 implementation selects the saved caller-stack register as CFA base
before its first source-stack pivot instruction. During abort recovery,
overwritten source-caller registers are unavailable until the pivot establishes
the owning caller's saved frame. An inactive abort cannot fabricate the
overwritten recovery register. These are debugger facts, not a runtime unwind
or exception service.

Between source steps the recovery cell is inactive while banks and controller
state remain live. Failure or cancellation there uses a separate target entry,
supplied by the owning controller with the same mapped exclusive temporary and
stack extents. It does not infer those extents from an inactive cell or trust
possibly inconsistent source-context/header fields. On the live controller
stack it preserves caller state, erases and reads back both entire extents,
clears bindings/registers and enters the same nonreturning fatal handler.
Geometry checks establish no ownership. Invalid controller protocol reaches
fatal handling without erasing unproven addresses or resuming the owner.
