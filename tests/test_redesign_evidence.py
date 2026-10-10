"""The requested review inventory and discriminating new evaluator checks."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]


def verifier(case):
    path = ROOT / 'evaluations/cases' / case / 'verify.py'
    spec = importlib.util.spec_from_file_location(case.replace('-', '_'), path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class SourceCoverageTests(unittest.TestCase):
    def test_every_requested_resource_has_pinned_path_level_evidence(self):
        repositories = '''walkinglabs/learn-harness-engineering nexu-io/harness-engineering-guide
openai/openai-agents-python affaan-m/ECC zhayujie/CowAgent pydantic/pydantic-ai
rohitg00/agentmemory zai-org/ZCode 1jehuang/jcode alibaba/open-code-review xai-org/grok-build
dream-num/univer mindfold-ai/Trellis nextlevelbuilder/ui-ux-pro-max-skill Nutlope/hallmark
Leonxlnx/taste-skill tt-a1i/archify emilkowalski/skills cathrynlavery/diagram-design
nexu-io/open-design pbakaus/impeccable remotion-dev/skills Graphify-Labs/graphify
obra/superpowers mattpocock/skills JuliusBrussee/caveman garrytan/gstack
mksglu/context-mode vercel-labs/agent-browser vercel-labs/agent-skills
addyosmani/agent-skills coreyhaines31/marketingskills sickn33/agentic-awesome-skills
ayghri/i-have-adhd OthmanAdi/planning-with-files zubair-trabzada/geo-seo-claude
anthropics/skills openai/skills google/skills openai/plugins'''.split()
        expected = {'https://github.com/' + name for name in repositories}
        expected.update(['https://developers.openai.com/mcp', 'https://learn.microsoft.com/api/mcp'])
        rows = []
        for name in ('harness', 'design', 'workflows', 'business-providers'):
            rows += json.loads((ROOT/'docs/research/harness-2026-10-10'/(name+'.json')).read_text(encoding='utf-8'))
        self.assertEqual(len(rows), 42)
        self.assertEqual({row['url'] for row in rows}, expected)
        for row in rows:
            with self.subTest(resource=row['id']):
                if row['url'].startswith('https://github.com/'):
                    self.assertRegex(row['revision'], r'^[0-9a-f]{40}$')
                else:
                    self.assertIn('2026-10-10', row['revision'])
                self.assertGreater(len(row['inspected_files']), 1)
                self.assertEqual(len(row['inspected_files']), len(set(row['inspected_files'])))
                for field in ('kind', 'decision', 'findings', 'risks', 'application'):
                    self.assertTrue(row[field])


class NewEvaluatorTests(unittest.TestCase):
    def test_financial_controls_distinguish_cash_profit_duplicates_and_delay(self):
        case = ROOT/'evaluations/cases/finance-cash-decision'
        data = json.loads((case/'workspace/ledger.json').read_text(encoding='utf-8'))
        oracle = verifier(case.name)
        result = oracle.expected(data)['scenarios']
        self.assertEqual([m['profit_cents'] for m in result['base']['months']], [2000000]*3)
        self.assertEqual([m['closing_cash_cents'] for m in result['base']['months']], [3000000,-2000000,0])
        self.assertEqual([m['closing_cash_cents'] for m in result['delayed']['months']], [3000000,-7000000,-5000000])
        self.assertEqual(result['base']['first_negative_period'], '2026-02')
        self.assertEqual(result['delayed']['financing_to_reserve_cents'], 8000000)
        data['transactions'].append(dict(data['transactions'][0], amount_cents=1))
        with self.assertRaises(ValueError):
            oracle.expected(data)
        self.assertFalse(oracle.same(True, 1))
        self.assertFalse(oracle.same({'cash':1}, {'cash':1, 'profit':2}))

    def test_financial_sparse_horizon_excludes_unlisted_cash_and_recognition(self):
        case = ROOT/'evaluations/cases/finance-cash-decision'
        data = json.loads((case/'workspace/ledger.json').read_text(encoding='utf-8'))
        data['periods'] = ['2026-01', '2026-03']
        result = verifier(case.name).expected(data)['scenarios']
        for name in ('base', 'delayed'):
            with self.subTest(scenario=name):
                months = result[name]['months']
                self.assertEqual([m['period'] for m in months], ['2026-01', '2026-03'])
                self.assertEqual([m['revenue_cents'] for m in months], [5000000, 5000000])
                self.assertEqual([m['operating_cost_cents'] for m in months], [3000000, 3000000])
                self.assertEqual([m['receipts_cents'] for m in months], [0, 5000000])
                self.assertEqual([m['payments_cents'] for m in months], [3000000, 3000000])
                self.assertEqual([m['closing_cash_cents'] for m in months], [3000000, 5000000])
                self.assertEqual(result[name]['minimum_cash_cents'], 3000000)
                self.assertEqual(result[name]['financing_to_reserve_cents'], 0)
                self.assertIsNone(result[name]['first_negative_period'])

    def test_localization_checks_reject_broken_contracts_without_claiming_fluency(self):
        oracle = verifier('localization-ui-contract')
        source = ROOT/'evaluations/cases/localization-ui-contract/workspace/en.json'
        good = {
            'refund': 'Tedarikçi iadeyi onaylayana kadar {booking_id} numaralı rezervasyonu iptal etmeyin. Talep, onaylanmış bir iade değildir.',
            'price': 'Tahmini toplam: 1.250,50 €. Yerel vergiler dahil değildir.',
            'deadline': '10 Ekim 2026 tarihinde saat 17:00 (Europe/Istanbul) itibarıyla onaylayın.',
            'seats': '{count, plural, =0 {Boş koltuk yok} one {# boş koltuk var} other {# boş koltuk var}}',
            'button': 'İptal talep et',
            'preview': '<strong>{name}</strong>, rezervasyonunuz beklemede, henüz onaylanmadı.'}
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root/'en.json').write_bytes(source.read_bytes())
            (root/'translation-qa.md').write_text('Fixture token checks only; not a native-human review.', encoding='utf-8')
            def failures(value):
                (root/'tr.json').write_text(json.dumps(value, ensure_ascii=False), encoding='utf-8')
                return {c['id'] for c in oracle.main(root) if c['status']=='failed'}
            self.assertEqual(failures(good), set())
            self.assertEqual(failures(dict(good, price='Tahmini toplam: €1.250,50. Yerel vergiler dahil değildir.')), set())
            self.assertEqual(oracle.substitutions("''#'' boş koltuk"), 1)
            self.assertEqual(oracle.substitutions("'#' boş koltuk"), 0)
            read_text = Path.read_text
            def windows_default(path, encoding=None, **kwargs):
                return read_text(path, encoding=encoding or 'cp1252', **kwargs)
            with mock.patch.object(Path, 'read_text', windows_default):
                self.assertEqual(failures(good), set())

            for key, value, check in (
                ('refund', good['refund'].replace('{booking_id}', '{id}'), 'protected-placeholders'),
                ('seats', good['seats'].replace('other', 'many'), 'icu-schema'),
                ('seats', good['seats'].replace('#', "'#'"), 'icu-schema'),
                ('seats', good['seats'].replace('# boş koltuk var', "'# boş koltuk var'"), 'icu-schema'),
                ('preview', good['preview'].replace('<strong>', '<b>'), 'markup'),
                ('price', 'Tahmini toplam: 1.250,50 TL.', 'amount-currency'),
                ('deadline', good['deadline'].replace('17:00', '18:00'), 'deadline')):
                altered = dict(good, **{key: value})
                self.assertIn(check, failures(altered))
            for key, value, check in (
                ('price', good['price'].replace('1.250,50', '11.250,50'), 'amount-currency'),
                ('price', good['price'] + ' Ön ödeme: 10 €.', 'amount-currency'),
                ('price', good['price'].replace('1.250,50', '-1.250,50'), 'amount-currency'),
                ('price', good['price'].replace('1.250,50', '− 1.250,50'), 'amount-currency'),
                ('deadline', good['deadline'].replace('2026', '20260'), 'deadline'),
                ('deadline', good['deadline'].replace('10 Ekim', '110 Ekim'), 'deadline'),
                ('deadline', good['deadline'].replace('17:00', '117:00'), 'deadline'),
                ('deadline', good['deadline'].replace('Europe/Istanbul', 'Asia/Europe/Istanbul'), 'deadline')):
                self.assertIn(check, failures(dict(good, **{key: value})))
            # Structural success deliberately does not certify translated meaning.
            altered = dict(good, refund='İadeyi beklemeden {booking_id} iptal edilebilir.')
            self.assertEqual(failures(altered), set())


if __name__ == '__main__':
    unittest.main()
