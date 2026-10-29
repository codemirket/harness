import json
import tempfile
import time
import unittest
from pathlib import Path
import review_oracles as review
import design_fixtures


class ReviewOracleTests(unittest.TestCase):
    def test_review_binds_actual_saved_artifact_rubric_and_inspection(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            snapshot = root / 'trials/cell/initial-project-snapshot'
            pack = root / 'reviewer/cell-initial'
            for path in (snapshot, pack, root/'reviews', root/'review-packs'):
                path.mkdir(parents=True)
            (snapshot/'art.svg').write_text('<svg/>')
            (pack/'art.svg').write_text('<svg/>')
            saved = review.tree(snapshot)
            rubric = 'Inspect saved geometry and report concrete defects.'
            template = dict(artifact_id='cell', phase='initial', manifest_sha256=review.canonical(saved), rubric_sha256=review.canonical(rubric))
            (pack/'review-template.json').write_text(json.dumps(template))
            write = lambda path, value: (root/path).write_text(json.dumps(value))
            write('trials/cell/initial-project-manifest.json', saved)
            write('review-packs/cell-initial.json', dict(manifest=review.tree(pack), created_at=time.time()-2))
            write('budget-ledger.json', dict(runs={'cell':{'initial':{'status':'finished','completed_at':time.time()-3}}}, deadline_at=time.time()+60))
            valid = dict(**template, status='reviewed', reviewed_by='independent-reviewer', reviewed_at=time.time()-1, inspected_artifacts=['art.svg'], findings=[])
            protocol={'cases':{'fixture':{'rubric':rubric}}}; cell={'id':'cell','task':'fixture'}
            write('reviews/cell.json',valid)
            self.assertEqual(review.validate_review(root,protocol,cell,'initial'),valid)
            for change in ({'status':'unreviewed'}, {'inspected_artifacts':[]}, {'manifest_sha256':'different'}, {'rubric_sha256':'different'}, {'findings':[{'severity':'major','id':'fake'}]}):
                write('reviews/cell.json', {**valid,**change})
                with self.assertRaises(SystemExit): review.validate_review(root,protocol,cell,'initial')
            write('reviews/cell.json',valid)
            (snapshot/'art.svg').write_text('<svg>changed</svg>')
            with self.assertRaisesRegex(SystemExit,'snapshot changed'): review.validate_review(root,protocol,cell,'initial')

    def test_creative_fixtures_keep_concrete_outputs_and_independent_rubrics(self):
        self.assertEqual(set(design_fixtures.CASES), {'product-page','illustration','video'})
        for case in review.CASES.values():
            self.assertTrue(case['outputs'])
            self.assertTrue(case['brief'])
            self.assertTrue(case['rubric'])
