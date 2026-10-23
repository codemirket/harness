import unittest
from maintenance import browser_proposal,node_proposal,release_proposal,snapshot_proposal
import datetime
class MaintenanceTests(unittest.TestCase):
    def test_new_security_release_produces_both_exact_architecture_pins(self):
        current={'version':'1.0','provenance':{},'archives':{'arm64':{},'amd64':{}}}
        stable={'version':'1.1','revision':'123','downloads':{'chrome':[{'platform':p,'url':'https://example/'+p} for p in ['linux-arm64','linux64']]}}
        proposal=browser_proposal(current,stable,lambda url:{'sha256':'a'*64,'bytes':123})
        self.assertEqual(proposal['after']['version'],'1.1');self.assertEqual(len(proposal['after']['archives']),2)
        self.assertTrue(proposal['owner']);self.assertIn('sandboxed design-check',proposal['gates'])
        stable['downloads']['chrome'].pop()
        with self.assertRaises(ValueError):browser_proposal(current,stable,lambda _: {})
    def test_node_does_not_jump_to_current_major(self):
        versions=[{'version':'v26.12.1','lts':False},{'version':'v24.22.2','lts':'Krypton','security':True}]
        p=node_proposal('old',versions,lambda tag:'sha256:'+'b'*64)
        self.assertEqual(p['version'],'24.22.2');self.assertTrue(p['security_release'])

    def test_native_update_is_concrete_and_prereleases_fail(self):
        release={'tag_name':'v1.2.3','html_url':'https://github.com/owner/project/releases/tag/v1.2.3','assets':[{'name':'linux-arm64.tar.gz','browser_download_url':'https://github.com/owner/project/releases/download/v1.2.3/linux-arm64.tar.gz','digest':'sha256:'+'a'*64}]}
        p=release_proposal('native','1.2.2',release,['runtime/native.json'])
        self.assertEqual(p['after'],'1.2.3');self.assertEqual(p['artifacts'][0]['digest'],'sha256:'+'a'*64)
        with self.assertRaises(ValueError):release_proposal('native','1.2.2',release|{'prerelease':True},[])
    def test_snapshot_proposal_never_moves_backwards(self):
        date=datetime.date(2026,10,23)
        self.assertIsNone(snapshot_proposal('20261024T000000Z',date))
        p=snapshot_proposal('20261001T000000Z',date)
        self.assertEqual(p['after'],'20261023T000000Z');self.assertTrue(p['owner'])
