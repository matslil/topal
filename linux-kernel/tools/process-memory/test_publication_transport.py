"""Reuse independent transport/checkpoint controls on the publication observer.

TOPAL-SYSTEMS-PROCESS-MM-001. These are host-only scheduling/record-publication
checks, not native memory or ownership evidence. Every selected request must
retain its real return stop.
"""
import json
from pathlib import Path
import sys
import tempfile

import test_interface_observer as base

sys.path.insert(0,str(Path(__file__).resolve().parents[2]/'labs/qemu/x86_64'))
from process_mm_publication_observer import CHECKPOINT_PROGRAM,PROGRAM


class PublicationTransportControls(base.InterfaceObserverControls):
    def setup_observer(self):
        temporary=tempfile.TemporaryDirectory();self.addCleanup(temporary.cleanup)
        folder=Path(temporary.name)
        scope=dict(json=json,observation_path=str(folder/'process-mm-observations.json'),
            publication_gateway_return=None,publication_active=None,publication_retiring=None)
        # A selected transport can have no publication (for example a break
        # query or preparation). Native publication/retirement checks have
        # their separate model and actual VM coverage.
        marker='pending=dict(request)'
        assert base.TRANSPORT.count(marker)==1
        transport=base.TRANSPORT.replace(marker,
            'pending=dict(request,operation=9,retired=[],before=dict(state=[0]*4),after=dict(state=[0]*4))')
        marker='class Gateway(Gateway):'
        assert PROGRAM.count(marker)==1
        scheduler=marker+PROGRAM.split(marker)[1].split('class Complete(Complete):')[0]
        exec(CHECKPOINT_PROGRAM+transport+scheduler,scope)
        enter=scope['Gateway']('entry',True)
        returned=scope['Gateway']('return',False)
        return folder,scope,enter,returned
