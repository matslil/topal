"""Retained owner counterexamples; model data, not injected guest execution."""
from copy import deepcopy
import unittest
from retained_image_tables import owner,retained_journal


def bank():
    return dict(base=0x3000000,bytes=4096,token=[1,2,0],
                record=[1,2,0,1,1,7,1,1],cells=[2,7,0x3000])


class RetainedOwner(unittest.TestCase):
    def test_unrelated_issuance_does_not_change_exact_retained_ownership(self):
        original=bank()
        for next_id in [3,4,1000]: owner(original,original,[1,1,next_id,100],True)

    def test_coherently_replaced_generation_is_not_the_retained_owner(self):
        original=bank(); changed=deepcopy(original)
        changed['record'][5]+=1;changed['cells'][1]+=1
        with self.assertRaises(ValueError): owner(changed,original,[1,1,1000,100],True)
        for key,index,value in [('token',0,2),('token',1,3),('record',6,0),('cells',2,0x3001)]:
            changed=deepcopy(original);changed[key][index]=value
            with self.subTest(key=key,index=index),self.assertRaises(ValueError):
                owner(changed,original,[1,1,1000,100],True)

    def test_release_preserves_generation_history_instead_of_rolling_it_back(self):
        original=bank(); released=deepcopy(original)
        released.update(token=[0,0,0],record=[0]*8,cells=[0,7,0x3000])
        owner(released,original,[1,1,1000,100],False)
        released['cells'][1]=0
        with self.assertRaises(ValueError): owner(released,original,[1,1,1000,100],False)

    def test_retirement_journal_distinguishes_old_and_current_context(self):
        first=dict(context=[1,1,5,100],controls=dict(cr3=0x101000),journal=[0,8],banks=[bank()]*3)
        retirement=dict(context=[1,1,1000,100],controls=dict(cr3=0x3100000))
        saved=retained_journal(first,None,1)
        current=retained_journal(first,retirement,2)
        self.assertEqual(saved[1:7],[1,5,0,0x101000,0,8])
        self.assertEqual(current[1:7],[1,5,1000,0x101000,0x3100000,8])
        self.assertEqual(saved[8:],current[8:])


if __name__=='__main__': unittest.main()
