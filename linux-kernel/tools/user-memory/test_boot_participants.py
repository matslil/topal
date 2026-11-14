"""TOPAL-SYSTEMS-BOOT-PARTICIPANT-ARRIVAL-001 model, never native evidence."""
from dataclasses import replace
from itertools import permutations
import unittest

from boot_participants import ArrivalFacts, Entry, Owner, Participant, Rendezvous, Session


def fixture(count=2):
    session = Session('provider', 'boot-domain', 'qualified-image', 1)
    owners = [Owner('code', 'boot', 1, 1, 0x1000, 0x1000, 'entry-code'),
              Owner('root', 'boot', 2, 1, 0x2000, 0x1000, 'root')]
    participants = []
    for i in range(count):
        labels = [f'{role}-{i}' for role in ('stack', 'mailbox', 'exceptions')]
        for role, label in zip(('stack', 'mailbox', 'exceptions'), labels):
            identity = len(owners) + 1
            owners.append(Owner(label, 'boot', identity, 1, identity * 0x1000, 0x1000, role))
        participants.append(Participant(f'cpu-{i}', 'enabled',
                                        Entry('code', 'root', *labels, 'qualified-entry-mode')))
    shared = ('code', 'root') if count > 1 else ()
    return session, tuple(participants), tuple(owners), shared


def make(count=2):
    session, participants, owners, shared = fixture(count)
    model = Rendezvous(session, 'cpu-0', participants, owners, shared)
    return model, session, participants, owners


def arrive(model, session, participant, owners):
    if participant.identity != 'cpu-0':
        model.dispatch(session, participant.identity)
        model.dispatch_result(session, participant.identity, True)
    model.enter(session, participant.identity, participant.entry, owners)
    model.acknowledge(session, participant.identity)


class BootParticipants(unittest.TestCase):
    def test_every_three_cpu_interleaving_matches_independent_prefix_oracle(self):
        # 8! event orders with immediate successful dispatch returns, with a
        # separate per-participant precedence oracle:
        # bootstrap E<A; each secondary D<E<A. Every invalid prefix is fatal;
        # every complete valid trace has all 3 acknowledgments and retains all
        # owners. No total CPU-order or synchronous-start assumption is used.
        events = [('enter', 0), ('ack', 0)] + [
            (kind, cpu) for cpu in (1, 2) for kind in ('dispatch', 'enter', 'ack')]
        accepted = 0
        for order in permutations(events):
            model, session, participants, owners = make(3)
            model.arm(owners, participants)
            seen = set()
            for kind, cpu in order:
                predecessor = ('dispatch', cpu) if kind == 'enter' and cpu else ('enter', cpu)
                valid = kind == 'dispatch' or kind == 'enter' and cpu == 0 or predecessor in seen
                participant = participants[cpu]
                if valid:
                    if kind == 'dispatch':
                        model.dispatch(session, participant.identity)
                        model.dispatch_result(session, participant.identity, True)
                    elif kind == 'enter':
                        model.enter(session, participant.identity, participant.entry, owners)
                    else:
                        model.acknowledge(session, participant.identity)
                    seen.add((kind, cpu))
                    self.assertEqual(set(model.pending),
                                     {f'cpu-{i}' for i in range(3) if ('ack', i) not in seen})
                else:
                    with self.assertRaises(ValueError):
                        if kind == 'enter':
                            model.enter(session, participant.identity, participant.entry, owners)
                        else:
                            model.acknowledge(session, participant.identity)
                    self.assertEqual(model.phase, 'fatal')
                    break
            else:
                facts = model.complete(owners, participants)
                self.assertEqual(facts, ArrivalFacts(session, ('cpu-0', 'cpu-1', 'cpu-2')))
                accepted += 1
            self.assertEqual(model.retained, owners)
        self.assertEqual(accepted, 560)  # 8! / (2! * 3! * 3!)

    def test_all_async_startup_return_interleavings_match_separate_oracle(self):
        events = ('b-enter', 'b-ack', 'dispatch', 'return', 'a-enter', 'a-ack')
        predecessors = {'b-enter': set(), 'b-ack': {'b-enter'}, 'dispatch': set(),
                        'return': {'dispatch'}, 'a-enter': {'dispatch'}, 'a-ack': {'a-enter'}}
        accepted = 0
        for order in permutations(events):
            model, session, participants, owners = make()
            model.arm(owners, participants)
            seen = set()
            actions = {
                'b-enter': lambda: model.enter(session, 'cpu-0', participants[0].entry, owners),
                'b-ack': lambda: model.acknowledge(session, 'cpu-0'),
                'dispatch': lambda: model.dispatch(session, 'cpu-1'),
                'return': lambda: model.dispatch_result(session, 'cpu-1', True),
                'a-enter': lambda: model.enter(session, 'cpu-1', participants[1].entry, owners),
                'a-ack': lambda: model.acknowledge(session, 'cpu-1'),
            }
            for event in order:
                if predecessors[event] <= seen:
                    actions[event]()
                    seen.add(event)
                else:
                    with self.assertRaises(ValueError):
                        actions[event]()
                    self.assertEqual(model.phase, 'fatal')
                    break
            else:
                model.complete(owners, participants)
                accepted += 1
            self.assertEqual(model.retained, owners)
        self.assertEqual(accepted, 45)

    def test_failed_missing_and_duplicate_dispatch_returns_retain_storage(self):
        for status in (False, 1, 'success', None):
            for after_ack in (False, True):
                model, session, participants, owners = make()
                model.arm(owners, participants)
                model.dispatch(session, 'cpu-1')
                if after_ack:
                    model.enter(session, 'cpu-1', participants[1].entry, owners)
                    model.acknowledge(session, 'cpu-1')
                with self.subTest(status=status, after_ack=after_ack), self.assertRaises(ValueError):
                    model.dispatch_result(session, 'cpu-1', status)
                self.assertEqual(model.phase, 'fatal')
                self.assertEqual(model.retained, owners)
        for duplicate in (False, True):
            model, session, participants, owners = make()
            model.arm(owners, participants)
            model.dispatch(session, 'cpu-1')
            for participant in participants:
                model.enter(session, participant.identity, participant.entry, owners)
                model.acknowledge(session, participant.identity)
            if duplicate:
                model.dispatch_result(session, 'cpu-1', True)
            with self.assertRaises(ValueError):
                if duplicate:
                    model.dispatch_result(session, 'cpu-1', True)
                else:
                    model.complete(owners, participants)
            self.assertEqual(model.phase, 'fatal')
            self.assertEqual(model.retained, owners)

    def test_dispatch_does_not_clear_pending_and_bootstrap_must_arrive(self):
        model, session, participants, owners = make()
        model.arm(owners, participants)
        model.dispatch(session, 'cpu-1')
        self.assertEqual(model.pending, ('cpu-0', 'cpu-1'))
        model.enter(session, 'cpu-1', participants[1].entry, owners)
        self.assertEqual(model.pending, ('cpu-0', 'cpu-1'))
        model.acknowledge(session, 'cpu-1')
        self.assertEqual(model.pending, ('cpu-0',))
        with self.assertRaisesRegex(ValueError, 'every participant'):
            model.complete(owners, participants)
        self.assertEqual(model.retained, owners)
        self.assertEqual(model.phase, 'fatal')

    def test_one_cpu_still_requires_actual_entry_and_acknowledgment(self):
        model, session, participants, owners = make(1)
        model.arm(owners, participants)
        arrive(model, session, participants[0], owners)
        model.complete(owners, participants)
        self.assertEqual(model.phase, 'arrived')
        self.assertEqual(model.retained, owners)

    def test_duplicate_dispatch_entry_and_ack_are_not_new_arrivals(self):
        for operation in ('dispatch', 'enter', 'ack'):
            with self.subTest(operation=operation):
                model, session, participants, owners = make()
                model.arm(owners, participants)
                model.dispatch(session, 'cpu-1')
                if operation != 'dispatch':
                    model.enter(session, 'cpu-1', participants[1].entry, owners)
                if operation == 'ack':
                    model.acknowledge(session, 'cpu-1')
                with self.assertRaises(ValueError):
                    if operation == 'dispatch':
                        model.dispatch(session, 'cpu-1')
                    elif operation == 'enter':
                        model.enter(session, 'cpu-1', participants[1].entry, owners)
                    else:
                        model.acknowledge(session, 'cpu-1')
                self.assertEqual(model.phase, 'fatal')
                self.assertEqual(model.retained, owners)

    def test_foreign_provider_domain_image_and_epoch_poison_live_session(self):
        for field, value in [('provider', 'other'), ('domain', 'other'),
                             ('image', 'other'), ('epoch', 2)]:
            for operation in ('dispatch', 'enter', 'ack'):
                with self.subTest(field=field, operation=operation):
                    model, session, participants, owners = make()
                    model.arm(owners, participants)
                    if operation != 'dispatch':
                        model.dispatch(session, 'cpu-1')
                    if operation == 'ack':
                        model.enter(session, 'cpu-1', participants[1].entry, owners)
                    foreign = replace(session, **{field: value})
                    with self.assertRaisesRegex(ValueError, 'foreign or stale'):
                        if operation == 'dispatch':
                            model.dispatch(foreign, 'cpu-1')
                        elif operation == 'enter':
                            model.enter(foreign, 'cpu-1', participants[1].entry, owners)
                        else:
                            model.acknowledge(foreign, 'cpu-1')
                    self.assertEqual(model.phase, 'fatal')
                    self.assertEqual(model.retained, owners)

    def test_wrong_participant_and_bootstrap_dispatch_are_rejected(self):
        for participant in ('unadvertised', 'cpu-0'):
            model, session, participants, owners = make()
            model.arm(owners, participants)
            with self.assertRaises(ValueError):
                model.dispatch(session, participant)
            self.assertEqual(model.phase, 'fatal')
            self.assertEqual(model.retained, owners)

    def test_each_target_local_binding_is_checked(self):
        for field in ('code', 'root', 'stack', 'mailbox', 'exceptions', 'mode'):
            model, session, participants, owners = make()
            model.arm(owners, participants)
            model.dispatch(session, 'cpu-1')
            wrong = replace(participants[1].entry, **{field: 'other'})
            with self.subTest(field=field), self.assertRaisesRegex(ValueError, 'target-local'):
                model.enter(session, 'cpu-1', wrong, owners)
            self.assertEqual(model.phase, 'fatal')
            self.assertEqual(model.retained, owners)
        model, session, participants, owners = make()
        model.arm(owners, participants)
        model.dispatch(session, 'cpu-1')
        with self.assertRaises(ValueError):
            model.enter(session, 'cpu-1', participants[0].entry, owners)

    def test_owner_changes_fail_at_entry_completion_and_later_check(self):
        mutations = dict(issuer='other', identity=99, generation=2, start=0x100000,
                         size=0x800, role='root', pins=2, live=False)
        for field, value in mutations.items():
            for boundary in ('entry', 'completion', 'later'):
                with self.subTest(field=field, boundary=boundary):
                    model, session, participants, owners = make()
                    model.arm(owners, participants)
                    changed = (replace(owners[0], **{field: value}),) + owners[1:]
                    if boundary != 'entry':
                        for participant in participants:
                            arrive(model, session, participant, owners)
                    if boundary == 'later':
                        model.complete(owners, participants)
                    with self.assertRaises(ValueError):
                        if boundary == 'entry':
                            model.enter(session, 'cpu-0', participants[0].entry, changed)
                        elif boundary == 'completion':
                            model.complete(changed, participants)
                        else:
                            model.check_lifetime(changed, participants)
                    self.assertEqual(model.phase, 'fatal')
                    self.assertEqual(model.retained, owners)

    def test_equal_looking_boolean_owner_fields_cannot_bypass_type_checks(self):
        for field in ('identity', 'generation', 'pins', 'live'):
            model, session, participants, owners = make()
            model.arm(owners, participants)
            changed = (replace(owners[0], **{field: 1 if field == 'live' else True}),) + owners[1:]
            # Dataclass equality would accept these equal-looking values.
            self.assertEqual(set(changed), set(owners))
            with self.subTest(field=field), self.assertRaises(ValueError):
                model.check_lifetime(changed, participants)
            self.assertEqual(model.phase, 'fatal')

    def test_changed_inventory_never_turns_an_unresponsive_cpu_into_absence(self):
        for boundary in ('arming', 'pending', 'arrived'):
            model, session, participants, owners = make()
            if boundary != 'arming':
                model.arm(owners, participants)
            if boundary == 'arrived':
                for participant in participants:
                    arrive(model, session, participant, owners)
                model.complete(owners, participants)
            with self.subTest(boundary=boundary), self.assertRaises(ValueError):
                if boundary == 'arming':
                    model.arm(owners, participants[:1])
                else:
                    model.check_lifetime(owners, participants[:1])
            self.assertEqual(model.phase, 'prepared' if boundary == 'arming' else 'fatal')
            self.assertEqual(model.retained, owners)

    def test_cancel_is_only_pre_arm_and_no_rearming_or_late_revival_exists(self):
        model, session, participants, owners = make()
        model.cancel()
        self.assertEqual(model.retained, ())
        for operation in (lambda: model.arm(owners, participants),
                          lambda: model.enter(session, 'cpu-0', participants[0].entry, owners)):
            with self.assertRaises(ValueError):
                operation()
            self.assertEqual(model.phase, 'cancelled')
        for after_dispatch in (False, True):
            model, session, participants, owners = make()
            model.arm(owners, participants)
            if after_dispatch:
                model.dispatch(session, 'cpu-1')
            with self.assertRaises(ValueError):
                model.cancel()
            self.assertEqual(model.phase, 'fatal')
            self.assertEqual(model.retained, owners)

    def test_timeout_retains_every_owner_despite_dispatch_or_partial_arrival(self):
        for progress in ('armed', 'dispatched', 'entered', 'acknowledged'):
            model, session, participants, owners = make()
            model.arm(owners, participants)
            if progress != 'armed':
                model.dispatch(session, 'cpu-1')
            if progress in ('entered', 'acknowledged'):
                model.enter(session, 'cpu-1', participants[1].entry, owners)
            if progress == 'acknowledged':
                model.acknowledge(session, 'cpu-1')
            model.timeout()
            first_failure = model.failure
            for operation in (lambda: model.enter(session, 'cpu-1', participants[1].entry, owners),
                              lambda: model.acknowledge(session, 'cpu-1'),
                              lambda: model.complete(owners, participants),
                              lambda: model.arm(owners, participants), lambda: model.cancel()):
                with self.subTest(progress=progress), self.assertRaises(ValueError):
                    operation()
                self.assertEqual(model.phase, 'fatal')
                self.assertEqual(model.failure, first_failure)
                self.assertEqual(model.retained, owners)

    def test_fault_after_arrival_keeps_only_historical_facts(self):
        model, session, participants, owners = make()
        model.arm(owners, participants)
        for participant in participants:
            arrive(model, session, participant, owners)
        facts = model.complete(owners, participants)
        model.fault(session, 'cpu-1')
        self.assertEqual(model.phase, 'fatal')
        self.assertEqual(model.retained, owners)
        self.assertEqual(facts.participants, ('cpu-0', 'cpu-1'))
        self.assertFalse(hasattr(facts, 'activate'))
        with self.assertRaises(ValueError):
            model.check_lifetime(owners, participants)

    def test_faults_at_each_entry_boundary_poison_complete_domain(self):
        for progress in ('armed', 'dispatched', 'entered', 'acknowledged'):
            model, session, participants, owners = make()
            model.arm(owners, participants)
            if progress != 'armed':
                model.dispatch(session, 'cpu-1')
            if progress in ('entered', 'acknowledged'):
                model.enter(session, 'cpu-1', participants[1].entry, owners)
            if progress == 'acknowledged':
                model.acknowledge(session, 'cpu-1')
            model.fault(session, 'cpu-1')
            self.assertEqual(model.phase, 'fatal')
            self.assertEqual(model.retained, owners)
            with self.assertRaises(ValueError):
                model.dispatch_result(session, 'cpu-1', True)

    def test_supporting_trampoline_is_whole_owned_and_retained(self):
        session, participants, owners, shared = fixture()
        trampoline = Owner('trampoline', 'boot', 99, 1, 0x80000, 0x1000, 'entry-code')
        participants = tuple(replace(p, entry=replace(p.entry, supporting_code=('trampoline',)))
                             for p in participants)
        owners += (trampoline,)
        model = Rendezvous(session, 'cpu-0', participants, owners, shared + ('trampoline',))
        model.arm(owners, participants)
        model.dispatch(session, 'cpu-1')
        changed = owners[:-1] + (replace(trampoline, generation=2),)
        with self.assertRaises(ValueError):
            model.enter(session, 'cpu-1', participants[1].entry, changed)
        self.assertEqual(model.phase, 'fatal')
        self.assertEqual(model.retained, owners)

    def test_preparation_refuses_incomplete_ambiguous_and_online_capable_inventory(self):
        session, participants, owners, shared = fixture()
        bad = [(), participants + participants[:1], participants[1:],
               (replace(participants[0], availability='online-capable'),) + participants[1:]]
        for changed in bad:
            with self.subTest(changed=changed), self.assertRaises(ValueError):
                Rendezvous(session, 'cpu-0', changed, owners, shared)

    def test_preparation_refuses_unpinned_aliased_partial_and_duplicate_owners(self):
        session, participants, owners, shared = fixture()
        bad = [owners[:-1], owners + owners[:1],
               (replace(owners[0], pins=0),) + owners[1:],
               (replace(owners[0], start=owners[1].start),) + owners[1:],
               (replace(owners[0], size=0),) + owners[1:],
               (replace(owners[0], start=(1 << 64) - 1),) + owners[1:],
               (replace(owners[0], identity=owners[1].identity, generation=2),) + owners[1:]]
        for changed in bad:
            with self.subTest(changed=changed), self.assertRaises(ValueError):
                Rendezvous(session, 'cpu-0', participants, changed, shared)

    def test_shared_root_code_are_explicit_and_private_entry_storage_is_never_shared(self):
        session, participants, owners, shared = fixture()
        for changed in ((), ('code',), ('code', 'root', 'extra'), ('code', 'root', 'root')):
            with self.subTest(shared=changed), self.assertRaises(ValueError):
                Rendezvous(session, 'cpu-0', participants, owners, changed)
        for field in ('stack', 'mailbox', 'exceptions'):
            entry = replace(participants[1].entry, **{field: getattr(participants[0].entry, field)})
            with self.subTest(field=field), self.assertRaises(ValueError):
                Rendezvous(session, 'cpu-0', participants[:1] + (replace(participants[1], entry=entry),), owners, shared)

    def test_owner_order_is_irrelevant_but_loss_and_extra_owners_fail(self):
        model, session, participants, owners = make()
        model.arm(tuple(reversed(owners)), participants)
        model.check_lifetime(tuple(reversed(owners)), participants)
        for changed in (owners[:-1], owners + owners[:1]):
            model, session, participants, owners = make()
            model.arm(owners, participants)
            with self.assertRaises(ValueError):
                model.check_lifetime(changed, participants)
            self.assertEqual(model.phase, 'fatal')

    def test_session_numbers_are_bounded_and_not_boolean(self):
        for epoch in (0, -1, True, 1 << 64):
            with self.subTest(epoch=epoch), self.assertRaises(ValueError):
                Session('provider', 'domain', 'image', epoch)
