"""TK-DEC-293 portable arrival model; no native CPU or allocation authority.

Inputs describe facts supplied by a hypothetical qualified provider. This model
checks their consistency and ordering, not their native truth or completeness.
In particular, constructing Owner/Entry/Session records cannot start a CPU,
create a language resource or prove image protection. No native caller exists.
"""
from dataclasses import dataclass


def require(condition, message):
    if not condition:
        raise ValueError(message)


def name(value):
    return type(value) is str and bool(value)


def natural(value):
    return type(value) is int and 0 < value < 1 << 64


@dataclass(frozen=True)
class Session:
    provider: str
    domain: str
    image: str
    epoch: int

    def __post_init__(self):
        require(all(name(x) for x in (self.provider, self.domain, self.image))
                and natural(self.epoch), 'complete session identity')


@dataclass(frozen=True)
class Owner:
    label: str
    issuer: str
    identity: int
    generation: int
    start: int
    size: int
    role: str
    pins: int = 1
    live: bool = True

    def validate(self):
        require(all(name(x) for x in (self.label, self.issuer, self.role)), 'named owner')
        require(all(natural(x) for x in (self.identity, self.generation, self.size, self.pins)),
                'live whole pinned owner identity')
        require(type(self.start) is int and 0 <= self.start < 1 << 64
                and self.start + self.size <= 1 << 64, 'bounded whole owner extent')
        require(self.live is True, 'owner must remain live')


@dataclass(frozen=True)
class Entry:
    code: str
    root: str
    stack: str
    mailbox: str
    exceptions: str
    mode: str
    supporting_code: tuple = ()

    def roles(self):
        require(type(self.supporting_code) is tuple and all(name(x) for x in self.supporting_code)
                and len(set(self.supporting_code + (self.code,))) == len(self.supporting_code) + 1,
                'distinct supporting entry-code owners')
        return (((self.code, 'entry-code'),) + tuple((x, 'entry-code') for x in self.supporting_code)
                + ((self.root, 'root'), (self.stack, 'stack'), (self.mailbox, 'mailbox'),
                   (self.exceptions, 'exceptions')))


@dataclass(frozen=True)
class Participant:
    identity: str
    availability: str
    entry: Entry


@dataclass(frozen=True)
class ArrivalFacts:
    """Historical modeled arrival only; deliberately no activation authority."""
    session: Session
    participants: tuple


class Rendezvous:
    """One finite session, without retry or post-arm storage reclamation.

    A native provider must issue fresh epochs and supply complete inventory,
    real ownership, actual target entry, publication/acquire events and clock
    evidence. The model cannot discover omitted facts or certify those inputs.
    """
    def __init__(self, session, bootstrap, participants, owners, shared=()):
        require(isinstance(session, Session), 'typed session')
        require(type(participants) is tuple and participants, 'complete finite inventory')
        require(all(isinstance(p, Participant) and name(p.identity)
                    and isinstance(p.entry, Entry) for p in participants), 'typed participants')
        require(len({p.identity for p in participants}) == len(participants), 'unique participants')
        require(name(bootstrap) and bootstrap in {p.identity for p in participants}, 'bootstrap included')
        require(all(p.availability == 'enabled' for p in participants),
                'online-capable or unsupported admission is unqualified')
        require(type(owners) is tuple and owners, 'complete retained owners')
        self._validate_owners(owners)
        by_label = {o.label: o for o in owners}
        references = set()
        private = set()
        uses = {}
        for participant in participants:
            entry = participant.entry
            require(name(entry.mode), 'qualified local mode description')
            for label, role in entry.roles():
                require(name(label) and label in by_label and by_label[label].role == role,
                        'entry bound to exact role owner')
                references.add(label)
                uses[label] = uses.get(label, 0) + 1
                if role in ('stack', 'mailbox', 'exceptions'):
                    require(label not in private, 'participant-private storage cannot be shared')
                    private.add(label)
        require(references == set(by_label), 'every retained owner has a declared entry role')
        require(type(shared) is tuple and all(name(label) for label in shared)
                and len(shared) == len(set(shared))
                and set(shared) == {label for label, count in uses.items() if count > 1},
                'exact explicit shared code/root contract')
        self._session = session
        self._bootstrap = bootstrap
        self._participants = participants
        self._owners = owners
        self._entries = {p.identity: p.entry for p in participants}
        self._states = {p.identity: 'prepared' for p in participants}
        self._requested = set()
        self._completed = set()
        self._phase = 'prepared'
        self._failure = None

    @staticmethod
    def _validate_owners(owners):
        require(all(isinstance(o, Owner) for o in owners), 'typed owners')
        for owner in owners:
            owner.validate()
        require(len({o.label for o in owners}) == len(owners), 'unique owner labels')
        # A live allocation ID is unique within an issuer, independently of
        # its generation. Generation cannot legitimize two simultaneous uses.
        require(len({(o.issuer, o.identity) for o in owners}) == len(owners),
                'unique live allocation identities')
        ordered = sorted(owners, key=lambda o: o.start)
        require(all(a.start + a.size <= b.start for a, b in zip(ordered, ordered[1:])),
                'complete retained owners must be physically disjoint')

    @property
    def phase(self):
        return self._phase

    @property
    def failure(self):
        return self._failure

    @property
    def pending(self):
        return tuple(p.identity for p in self._participants
                     if self._states[p.identity] != 'acknowledged')

    @property
    def retained(self):
        # Cancellation describes return to the caller, not physical free. No
        # allocator effects or preexisting owner's validity are modeled here.
        return () if self._phase == 'cancelled' else self._owners

    def _check(self, condition, message):
        if not condition:
            if self._phase in ('armed', 'arrived'):
                self._phase = 'fatal'
                self._failure = message
            raise ValueError(message)

    def _live(self):
        self._check(self._phase in ('armed', 'arrived'), 'session is not live')

    def _binding(self, session, participant):
        self._live()
        self._check(isinstance(session, Session) and session == self._session,
                    'foreign or stale session')
        self._check(name(participant) and participant in self._states, 'unexpected participant')

    def _retained_equal(self, owners):
        self._check(type(owners) is tuple, 'typed retained owner sequence')
        try:
            self._validate_owners(owners)
        except ValueError as error:
            self._check(False, f'invalid retained owners: {error}')
        self._check(len(owners) == len(self._owners) and set(owners) == set(self._owners),
                    'retained owner binding changed')

    def arm(self, owners, participants):
        self._check(self._phase == 'prepared', 'arming requires prepared state')
        self._retained_equal(owners)
        self._check(participants == self._participants, 'complete inventory changed')
        # This precedes publication/startup. No possible native effect may be
        # issued before the model has retained every pending obligation.
        self._states = dict.fromkeys(self._states, 'pending')
        self._phase = 'armed'

    def dispatch(self, session, participant):
        self._binding(session, participant)
        self._check(self._phase == 'armed' and participant != self._bootstrap,
                    'dispatch requires an armed secondary participant')
        self._check(participant not in self._requested and self._states[participant] == 'pending',
                    'one startup request per participant')
        self._requested.add(participant)

    def dispatch_result(self, session, participant, success):
        self._binding(session, participant)
        self._check(self._phase == 'armed' and participant in self._requested
                    and participant not in self._completed, 'one result for a dispatched request')
        self._check(success is True, 'startup request failed or result is unqualified')
        self._completed.add(participant)

    def enter(self, session, participant, local_entry, owners):
        self._binding(session, participant)
        self._check(self._phase == 'armed' and self._states[participant] == 'pending',
                    'entry requires its pending participant')
        self._check(participant == self._bootstrap or participant in self._requested,
                    'secondary entry requires its dispatched request')
        self._retained_equal(owners)
        self._check(local_entry == self._entries[participant], 'target-local entry binding changed')
        self._states[participant] = 'entered'

    def acknowledge(self, session, participant):
        self._binding(session, participant)
        self._check(self._phase == 'armed' and self._states[participant] == 'entered',
                    'acknowledgment requires its validated target entry')
        # The event describes target release + coordinator acquire, not a
        # mailbox integer written by the coordinator or observed in a debugger.
        self._states[participant] = 'acknowledged'

    def complete(self, owners, participants):
        self._live()
        self._check(self._phase == 'armed' and not self.pending, 'every participant must acknowledge')
        self._check(self._requested == self._completed, 'every startup request must complete successfully')
        self._retained_equal(owners)
        self._check(participants == self._participants, 'complete inventory changed')
        self._phase = 'arrived'
        return ArrivalFacts(self._session, tuple(p.identity for p in self._participants))

    def check_lifetime(self, owners, participants):
        self._live()
        self._retained_equal(owners)
        self._check(participants == self._participants, 'complete inventory changed')

    def timeout(self):
        self._live()
        self._check(self._phase == 'armed', 'arrival deadline applies only before completion')
        self._phase = 'fatal'
        self._failure = 'arrival deadline expired; delayed entry remains possible'

    def fault(self, session, participant):
        self._binding(session, participant)
        self._phase = 'fatal'
        self._failure = 'participant fault; arrival cannot supply continuous protection'

    def cancel(self):
        self._check(self._phase == 'prepared', 'cancellation after arming cannot reclaim entry storage')
        self._phase = 'cancelled'
