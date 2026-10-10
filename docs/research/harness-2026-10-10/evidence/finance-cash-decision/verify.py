"""Public development oracle: checks arithmetic, never financial judgment."""
import copy
import json
from pathlib import Path
import subprocess
import sys
import tempfile


def expected(data):
    unique = {}
    for row in data['transactions']:
        if row['id'] in unique and unique[row['id']] != row:
            raise ValueError('Conflicting transaction')
        unique[row['id']] = row
    scenarios = {}
    for scenario in ('base', 'delayed'):
        cash = data['opening_cash_cents']
        balances, months = [cash], []
        for period in data['periods']:
            revenue = costs = receipts = payments = 0
            for row in unique.values():
                kind, amount = row['kind'], row['amount_cents']
                if row['recognized_period'] == period:
                    revenue += amount if kind == 'invoice' else 0
                    costs += amount if kind == 'operating_cost' else 0
                due = row['cash_period']
                if scenario == 'delayed' and kind == 'invoice':
                    year, month = map(int, due.split('-'))
                    due = f'{year + (month == 12):04d}-{month % 12 + 1:02d}'
                if due == period:
                    receipts += amount if kind == 'invoice' else 0
                    payments += amount if kind != 'invoice' else 0
            cash += receipts - payments
            balances.append(cash)
            months.append(dict(period=period, revenue_cents=revenue, operating_cost_cents=costs,
                               profit_cents=revenue-costs, receipts_cents=receipts,
                               payments_cents=payments, closing_cash_cents=cash))
        minimum = min(balances)
        scenarios[scenario] = dict(months=months, minimum_cash_cents=minimum,
                                   first_negative_period=next((r['period'] for r in months
                                                              if r['closing_cash_cents'] < 0), None),
                                   financing_to_reserve_cents=max(0, data['minimum_reserve_cents']-minimum))
    return dict(currency=data['currency'], scenarios=scenarios)


def same(actual, wanted):
    if type(actual) is not type(wanted):
        return False
    if isinstance(wanted, dict):
        return actual.keys() == wanted.keys() and all(same(actual[k], v) for k, v in wanted.items())
    if isinstance(wanted, list):
        return len(actual) == len(wanted) and all(same(a, b) for a, b in zip(actual, wanted))
    return actual == wanted


def main(root):
    checks = []
    def check(name, passed, detail):
        checks.append(dict(id=name, status='passed' if passed else 'failed', detail=detail))
    source = root / 'analyze.py'
    if not source.is_file():
        return [dict(id='program', status='failed', detail='Missing analyze.py')]
    fixture = json.loads((root / 'ledger.json').read_text(encoding='utf-8'))
    control = expected(fixture)
    check('independent-controls',
          [m['closing_cash_cents'] for m in control['scenarios']['base']['months']] == [3000000,-2000000,0]
          and control['scenarios']['base']['financing_to_reserve_cents'] == 3000000
          and control['scenarios']['delayed']['financing_to_reserve_cents'] == 8000000,
          'Hand-calculated EUR cash controls and reserve financing match')
    with tempfile.TemporaryDirectory(prefix='finance-verifier-') as tmp:
        scratch = Path(tmp)
        def run(name, data):
            path, output = scratch / (name+'.json'), scratch / (name+'-result.json')
            path.write_text(json.dumps(data), encoding='utf-8')
            result = subprocess.run([sys.executable, '-I', str(source), '--input', str(path), '--output', str(output)],
                                    cwd=root, capture_output=True, timeout=5)
            return result, output
        variants = [('fixture', fixture)]
        altered = copy.deepcopy(fixture)
        altered['opening_cash_cents'] = 15000000
        altered['transactions'].reverse()
        variants.append(('funded-reordered', altered))
        altered = copy.deepcopy(fixture)
        altered['periods'] = ['2026-12', '2027-01', '2027-02']
        mapping = dict(zip(['2026-01','2026-02','2026-03','2026-04'],
                           ['2026-12','2027-01','2027-02','2027-03']))
        for row in altered['transactions']:
            row['cash_period'] = mapping[row['cash_period']]
            row['recognized_period'] = mapping.get(row['recognized_period'])
            row['amount_cents'] += 317
        variants.append(('year-boundary', altered))
        for name, data in variants:
            try:
                result, output = run(name, data)
                actual = json.loads(output.read_text(encoding='utf-8')) if output.exists() else None
                check(name, result.returncode == 0 and same(actual, expected(data)),
                      'Actual input-derived values, types and scenario contracts')
            except Exception as exc:
                check(name, False, type(exc).__name__ + ': ' + str(exc)[:160])
        bad = copy.deepcopy(fixture)
        bad['transactions'].append(dict(bad['transactions'][0], amount_cents=1))
        try:
            result, output = run('conflict', bad)
            check('conflicting-id', result.returncode != 0 and not output.exists(),
                  'Conflicting record does not silently become a valid financial report')
        except Exception as exc:
            check('conflicting-id', False, str(exc)[:160])
    try:
        check('saved-output', same(json.loads((root/'results.json').read_text(encoding='utf-8')), control),
              'Delivered output matches independently reconstructed source values')
    except Exception as exc:
        check('saved-output', False, str(exc)[:160])
    note = root/'decision.md'
    check('decision-artifact', note.is_file() and bool(note.read_text(encoding='utf-8').strip()),
          'Decision artifact exists; its substance requires separate qualitative review')
    return checks


if __name__ == '__main__':
    checks = main(Path(sys.argv[1]).resolve())
    print(json.dumps({'checks': checks}))
    sys.exit(0 if all(c['status']=='passed' for c in checks) else 1)
