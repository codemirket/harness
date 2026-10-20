import unittest
import io
import json
import tarfile
import tempfile
from pathlib import Path
from image_contents import overlay, name, contents

class OverlayTests(unittest.TestCase):
    def test_final_image_root_whiteout_cannot_inherit_removed_payload(self):
        def layer(entries):
            buffer = io.BytesIO()
            with tarfile.open(fileobj=buffer,mode='w') as archive:
                for filename, data in entries:
                    member = tarfile.TarInfo(filename);member.size=len(data)
                    archive.addfile(member,io.BytesIO(data))
            return buffer.getvalue()
        with tempfile.TemporaryDirectory() as temporary:
            image = Path(temporary)/'image.tar'
            payloads = {'manifest.json':json.dumps([{'Layers':['a.tar','b.tar']}]).encode(),
                        'a.tar':layer([('opt/app/old',b'old')]),
                        'b.tar':layer([('.wh..wh..opq',b''),('opt/app/new',b'new')])}
            with tarfile.open(image,'w') as archive:
                for filename,data in payloads.items():
                    member=tarfile.TarInfo(filename);member.size=len(data)
                    archive.addfile(member,io.BytesIO(data))
            self.assertEqual(set(contents(image,['opt/app'])),{'opt/app/new'})
    def test_whiteouts_never_restore_deleted_old_inventory(self):
        old = {'var/lib/dpkg/status':{'type':'file','sha256':'old'}}
        self.assertEqual(overlay(old,[('var/.wh.lib',{'type':'file'})]),{})
        self.assertEqual(overlay(old,[('var/lib/.wh..wh..opq',{'type':'file'})]),{})
        fresh = {'type':'file','sha256':'new'}
        self.assertEqual(overlay(old,[('var/lib/dpkg/status',fresh),
                                     ('var/lib/.wh..wh..opq',{'type':'file'})]),
                         {'var/lib/dpkg/status':fresh})
    def test_link_is_compared_without_reading_its_target(self):
        link = {'type':'symlink','target':'/private/file','mode':511}
        self.assertEqual(overlay({'opt/app/old':{'type':'file'}},[('opt/app',link)]),{'opt/app':link})
        for invalid in ['/etc/passwd','../secret','opt/../../secret']:
            with self.assertRaises(ValueError): name(invalid)
