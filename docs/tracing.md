# Tracing

Compiled Topal applications and libraries contain trace support by default.
The compiler emits dormant trace sites, stable event identities, type
definitions, and native serialization functions. An external tracing tool
selects events, controls collection, receives the resulting
[`SerializationStream`](serialization.md), and chooses how to encode or store
it.

Application code is unaware of tracing. It cannot discover whether a tool is
connected, inspect trace configuration, observe channel rejection, enable
events, or change authorization policy. Tracing may affect elapsed execution
time but cannot change application values, failures, or control flow.

## Purposes and profiles

One trace stream may serve several purposes. Its envelope therefore contains a
`profiles` collection rather than one exclusive profile. The initial profiles
are `debugging` and `testing`. Format is independent: a native, CTF, Google
Trace Event, or other adapter encodes the same selected semantic stream.

Debugging events are authoritative when profiles overlap. Enabling `testing`
adds decision evidence, such as binding and candidate-selection events, but
does not emit a second copy of a debugging event. An interpreter test run uses
at least `( debugging, testing )`; an ordinary debugger uses `debugging` and may
select additional profiles.

## Fundamental debugging events

The language-defined debugging foundation contains only these event cases:

- `lang ValueEvent create`, when a semantic value begins to exist;
- `lang ValueEvent destroy`, when that semantic value's lifetime ends;
- `lang ValueEvent access`, when that value is read or supplied to a function;
- `lang FunctionEvent entry`, immediately before a function executes; and
- `lang FunctionEvent exit`, after its result exists and before control returns.

An operator is a function supplied by the core language and uses the same
`entry` and `exit` events. Create and destroy describe semantic lifetime, not
allocation and deallocation. Access does not expose whether the implementation
borrowed, moved, copied, or shared a representation; a debugger may display
that decision as additional evidence after stopping.

Bindings are aliases and do not add a fundamental debugging event. The testing
profile may record `bind` evidence to verify name resolution and execution
decisions. Resources, messages, scheduler transitions, function failure, and
transaction states are not extra fundamental kinds. Typed higher-level events
derive them from value and function events. Compiler-owned hidden functions
use ordinary structured Topal identities, for example `lang task switch-to`,
so they remain distinguishable from a source-defined `switch-to`.

## Introspective trace observers

`lang trace` constructs a constrained observational task. Its typed inputs
declare the fundamental or already-derived events and statically supplied
configuration values it may inspect. Its handlers use ordinary Topal
conditions, matches, functions, task state, enums, and unions. Returning an
event enum value publishes that typed value as a derived trace event; returning
`None` publishes nothing.

The task analogy defines event delivery, private recognition state, and
ordering. It does not require a physical task. An implementation may lower a
stateless observer to a filter or a stateful observer to an optimized state
machine. Equivalent input streams and initial configurations must produce the
same derived stream.

Observer introspection is non-observing: inspecting an event argument does not
itself produce `access`, `entry`, or `exit`. An observer cannot mutate or send a
message to the observed application, change its scheduling or result, acquire
application authority, or observe whether an adapter is attached. Its state
depends only on its configuration and ordered input events. Derived-observer
dependencies are acyclic; loss invalidating stateful recognition is reported.

The emitted event's enum type supplies its event group, its alternative
supplies the event kind, and its associated value supplies the payload. The
stream adds sequence, provenance, source location, task, execution-lane,
active-profile, language-version, and schema information. Adapters decide
whether to store, display, count, or break on the event.

Source locations, qualified function identities, type identities, and similar
typed static values may configure observers. A debugger can consequently
instantiate an observer which derives an event when execution reaches a
command-line-selected source location. Declaring such a reusable observer in
ordinary source is harmless: it remains dormant unless selected by authorized
trace control.

## Compiled support

Every compiled artifact normally contains:

- a catalog of its trace providers, event identities, and payload types;
- type definitions needed to describe those payloads;
- dormant instrumentation at language-defined and explicitly declared trace
  sites;
- specialized native serialization functions for statically known payload
  types; and
- the runtime adapter for an optional control and data channel.

Libraries contribute their catalog and trace sites to the final application.
Linking combines them without changing their stable semantic identities. The
final application supplies the runtime channel and authorization policy;
libraries cannot create independent control surfaces or weaken that policy.

When no event is enabled, a trace site performs only a predictable disabled
check. Payloads are not serialized, copied, allocated, or evaluated solely for
tracing. The compiler may combine checks, remove unreachable sites, and
specialize enabled paths while preserving the defined event identities.

A specialized build may explicitly remove trace support. This is not the
default, and artifact metadata records whether the support and catalog are
present.

## External control

Trace selection and collection belong to an external tool rather than the
application. The tool may request:

```text
Catalog
  providers, event identities, and payload type definitions
  no event values

Collect
  a selected set of event identities
  their payload type definitions
  a stream of event values
```

Catalog mode lets tooling present the available trace surface without enabling
any trace site. Selection may use human-facing provider, module, function, and
event names, but the runtime receives resolved stable identities rather than
matching text on every event.

Changing a selection affects subsequent occurrences. It does not synthesize
events which happened before activation. The control protocol acknowledges the
selection point so the tool can distinguish events which were not enabled from
events lost after enablement.

## Launch collection

To collect from the first application event, the tool:

1. creates and secures the trace channel;
2. launches the application with bootstrap information identifying that
   channel;
3. completes authorization and protocol negotiation;
4. requests the catalog or selected events; and
5. releases application startup.

The trace runtime performs this handshake before root-task initialization or
any other application event. Failure policy belongs to the launcher. It may
terminate a launch which explicitly required complete tracing or allow the
application to continue with all sites disabled.

The application itself receives no result from this process and cannot select
between those policies.

## Later attachment

An application can accept a tracing tool after startup only when its process
was launched with an OS-specific trace channel or rendezvous endpoint. An
environment variable may identify that bootstrap channel:

```text
TOPAL_TRACE_CHANNEL=<platform-specific channel description>
```

The name above describes the conventional role; exact spelling and contents
belong to the target runtime. The variable is consumed by runtime bootstrap and
is not supplied as a Topal [context constructor](contexts.md) argument or made
visible to application code.

When no channel is configured, the runtime creates no attach endpoint and later
attachment is impossible. Compiled trace sites remain dormant. Merely compiling
trace support into an application does not expose a tracing service.

When a channel is configured, its minimal control endpoint may remain available
while no events are enabled. A later tool can authenticate, request the catalog,
select events, and begin receiving subsequent values.

The transport is target-specific. Implementations may use inherited
descriptors, Unix-domain sockets, Windows named pipes, shared-memory data rings
with an authenticated control channel, platform tracing services, or sandbox
capabilities. Network endpoints are not enabled by the default policy.

## Stream ownership

The application produces native Topal serialization. The tool owns external
encoding, transport beyond the trace channel, and storage:

```text
compiled trace sites
        |
        v
native SerializationStream
        |
        v
external tracing tool
        |
        +--> encode CTF
        +--> encode another trace format
        +--> relay native stream
        `--> store or rotate output
```

The source model retains a serialization step followed by an encoding step.
An implementation may fuse them so a CTF collector receives metadata and
native-endian event data without an intermediate native byte buffer.

The application does not choose CTF, open trace files, rotate logs, compress
blocks, or select network destinations. Those effects and their failures remain
outside application semantics.

## Containment and loss

Tracing follows the containment guarantees required of
[diagnostic capabilities](contexts.md#capabilities-endpoints-and-containment):

- delivery failure cannot escape into application code;
- the trace channel cannot invoke an application callback;
- trace state cannot modify application-owned values;
- application execution never waits indefinitely for a collector;
- buffering and retry work have declared finite bounds; and
- failures do not alter a function's semantic result.

Ordinary collection uses bounded, nonblocking event publication. Buffer
exhaustion drops events and records a lost-event count in later stream or packet
context. A tool can therefore distinguish an event which was disabled from one
which was enabled but lost.

A deployment may explicitly run in a trace-constrained execution mode which
permits bounded suspension to obtain stronger delivery guarantees. That mode is
an execution policy selected by the launcher, not an observation available to
application code.

## Compiled authorization policy

The final application build embeds the maximum authority accepted by its trace
runtime. This is target and deployment configuration rather than Topal source
semantics. A conceptual compiler interface may accept:

```text
--trace-authority default
--trace-authority user:<identity>
--trace-authority group:<identity>
--trace-authority inherited-capability
```

The setting records public identities and authorization rules, never passwords,
tokens, or private keys. Runtime channel configuration may restrict the
compiled policy further but cannot weaken it:

```text
effective authorization
  = compiled maximum policy
  intersect channel permissions
  intersect authenticated peer identity
```

Linked libraries contribute no authorization settings. The final application
policy governs every linked trace site.

Artifact metadata exposes the policy needed by deployment tooling without
exposing secrets:

```text
trace support          present
bootstrap channel      required
authorized authority   platform default
network tracing        prohibited
```

## Unix default

For a filesystem-backed Unix channel, the secure default requires:

```text
owner  application effective user or root
group  topaltrace
mode   0660
ACL    no additional principals
```

The endpoint and every containing directory are validated. The runtime rejects
symlinks, an unexpected channel kind, replacement by an unauthorized user,
access granted to `other`, and ACL entries outside the compiled policy. It also
proves that the application can actually use the endpoint. A root-owned
endpoint consequently requires application access through the tracing group,
an explicit permitted ACL, or an already inherited capability.

Filesystem metadata controls access to the endpoint but does not authenticate
the connected process by itself. A connected Unix socket additionally uses
platform peer credentials. Under the default policy, the peer user is the
application's effective user or root and the peer uses the `topaltrace`
authority. Where supplementary groups cannot be authenticated reliably, the
tool uses `topaltrace` as its effective group or connects through an
authenticated tracing broker.

An inherited descriptor prevents pathname substitution and discovery races but
does not inherently prove who owns its peer. The runtime verifies the
descriptor kind and connected peer credentials. A compiler policy may instead
declare possession of an inherited descriptor to be the authority when the
deployment explicitly trusts its launcher. Anonymous pipes and ordinary files
which cannot authenticate a peer do not satisfy a user- or group-based policy.

## Other platform defaults

Each target maps the same roles to its native security model:

```text
application principal
TopalTrace tracing principal
system administrator principal
no other access
```

Windows implementations use restricted handle inheritance or named-pipe ACLs
containing the application SID, the configured TopalTrace group SID, and
intended system-administrator identities. Broad principals such as `Everyone`
do not satisfy the default. The connected peer token is authenticated in
addition to checking the channel ACL.

Sandboxed macOS, Android, and iOS implementations may use entitled brokers,
application identities, platform permissions, or mandatory-access-control
domains rather than a literal Unix group. A target which cannot safely provide
later attachment supports only launch-time inherited or brokered tracing. The
semantic authorization roles and absence of application awareness remain the
same.

## Rejection

An absent, inaccessible, malformed, or unauthorized channel leaves every trace
site disabled. Application code receives no status and observes no error.

When the runtime can communicate over the proposed channel, it may send one
coarse rejection before closing:

```text
WrongOwner
WrongGroup
AccessTooBroad
ApplicationAccessMissing
PeerUnauthorized
InsecureDirectory
UnsupportedChannel
UnsupportedProtocol
```

The response contains no paths, numeric identities, permission bits, ACL
contents, credentials, or expected values. A channel which cannot be opened or
used receives no response. The external tool can still diagnose the endpoint
it created using its own platform access.

Because a rejected channel is not trusted, its rejection is informational
rather than an authenticated application message. Repeated attempts are
bounded or rate-limited. Rejection never uses application logging, never enters
the native trace event stream, and is not visible through `lang` introspection.

The resulting application behavior is:

```text
no channel configured       -> tracing disabled
valid authorized channel    -> tracing controlled by the tool
usable but invalid channel  -> coarse rejection, then tracing disabled
unusable channel            -> tracing disabled without response
```

## Initial scope

The initial tracing facility should define:

- default instrumentation and catalogs for applications and libraries;
- stable provider, event, and payload identities;
- catalog-only and selected-event collection modes;
- launch-time first-event collection;
- opt-in later attachment through an OS bootstrap channel;
- native `SerializationStream` transport to an external tool;
- bounded buffers, lost-event accounting, and contained failure;
- compiler-configured maximum authorization with secure platform defaults;
- authenticated local channels for the initial target platforms; and
- coarse in-band rejection where the proposed channel is usable.

It should not initially define:

- application-visible trace status or control;
- tracing configured from Topal context constructor arguments;
- a default network tracing endpoint;
- one universal transport for every operating system;
- unbounded lossless collection;
- external trace-file management in the application; or
- authorization settings supplied by linked libraries.
