"""Public development checks: deterministic arithmetic, not decision quality."""
import csv
import json
import math
import random
import subprocess
import sys
import tempfile
from pathlib import Path

COHORTS = ('current', 'pilot')
DEVICES = ('mobile', 'desktop')


def expected(accounts, deliveries):
    """Independent account-set oracle; shared shape comes from the task contract."""
    by_id = {row['account_id']: row for row in accounts}
    unique = {}
    excluded = dict.fromkeys(('duplicate_deliveries', 'unknown_accounts', 'ineligible_accounts', 'non_paid'), 0)
    buyers = set()
    for row in deliveries:
        key = row['order_id']
        if key in unique:
            if row != unique[key]:
                raise ValueError('conflicting order')
            excluded['duplicate_deliveries'] += 1
        else:
            unique[key] = row
    for row in unique.values():
        account = by_id.get(row['account_id'])
        if account is None:
            excluded['unknown_accounts'] += 1
        elif str(account['eligible']) != '1':
            excluded['ineligible_accounts'] += 1
        elif row['status'] != 'paid':
            excluded['non_paid'] += 1
        else:
            buyers.add(row['account_id'])

    def metric(cohort, device=None):
        ids = {r['account_id'] for r in accounts if str(r['eligible']) == '1'
               and r['cohort'] == cohort and (device is None or r['device'] == device)}
        numerator = len(ids & buyers)
        return {'accounts': len(ids), 'converted_accounts': numerator,
                'conversion_rate': numerator / len(ids) if ids else None}

    cohorts = {c: metric(c) for c in COHORTS}
    strata = {d: {c: metric(c, d) for c in COHORTS} for d in DEVICES}
    weights = {d: strata[d]['current']['accounts'] / cohorts['current']['accounts'] for d in DEVICES}
    rates = {}
    for c in COHORTS:
        rates[c] = (None if any(weights[d] and strata[d][c]['conversion_rate'] is None for d in DEVICES)
                    else sum(weights[d] * (strata[d][c]['conversion_rate'] or 0) for d in DEVICES))
    return {'cohorts': cohorts, 'by_device': strata,
            'standardized': {'weights': weights, 'current_rate': rates['current'], 'pilot_rate': rates['pilot'],
                             'difference_pp': None if rates['pilot'] is None else 100 * (rates['pilot'] - rates['current'])},
            'excluded_orders': excluded}


def equivalent(actual, wanted):
    if isinstance(wanted, dict):
        return isinstance(actual, dict) and set(actual) == set(wanted) and all(equivalent(actual[k], v) for k, v in wanted.items())
    if wanted is None:
        return actual is None
    if isinstance(wanted, int):
        return type(actual) is int and actual == wanted
    return type(actual) in (int, float) and math.isfinite(actual) and math.isclose(actual, wanted, abs_tol=1e-9, rel_tol=1e-9)


def rows(path):
    with path.open(newline='', encoding='utf-8') as stream:
        return list(csv.DictReader(stream))


def write_data(path, accounts, orders):
    path.mkdir()
    for name, fields, values in [('accounts.csv', ('account_id', 'cohort', 'device', 'eligible'), accounts),
                                 ('orders.csv', ('order_id', 'account_id', 'status'), orders)]:
        with (path / name).open('w', newline='', encoding='utf-8') as stream:
            writer = csv.DictWriter(stream, fieldnames=fields)
            writer.writeheader()
            writer.writerows(values)


def generated(seed, missing_device=False):
    rng = random.Random(seed)
    accounts, orders = [], []
    for c in COHORTS:
        for d in DEVICES:
            count = 0 if missing_device and c == 'pilot' and d == 'desktop' else rng.randint(2, 9)
            for i in range(count):
                aid = f'{seed}-{c}-{d}-{i}'
                accounts.append(dict(account_id=aid, cohort=c, device=d, eligible='1'))
                for j in range(rng.randint(0, 3)):
                    orders.append(dict(order_id=f'{aid}-{j}', account_id=aid, status=rng.choice(('paid', 'paid', 'failed', 'cancelled'))))
    accounts.append(dict(account_id=f'excluded-{seed}', cohort='pilot', device='desktop', eligible='0'))
    orders.extend([dict(order_id='unknown', account_id='absent', status='paid'),
                   dict(order_id='ineligible', account_id=f'excluded-{seed}', status='failed')])
    orders += [dict(orders[0]), dict(orders[-1])]
    rng.shuffle(accounts)
    rng.shuffle(orders)
    return accounts, orders


def main(root):
    checks = []

    def check(name, passed, detail):
        checks.append({'id': name, 'status': 'passed' if passed else 'failed', 'detail': detail})

    source = root / 'analyze.py'
    if not source.is_file():
        return [{'id': 'program', 'status': 'failed', 'detail': 'analyze.py missing'}]

    def run(data, output):
        result = subprocess.run([sys.executable, '-I', str(source), '--data', str(data), '--output', str(output)],
                                cwd=root, capture_output=True, timeout=5)
        if result.returncode or not output.is_file():
            raise ValueError(f'Analysis failed or omitted output (exit {result.returncode})')
        return json.loads(output.read_text(encoding='utf-8'))

    with tempfile.TemporaryDirectory(prefix='rollout-eval-') as temporary:
        scratch = Path(temporary)
        try:
            fixture = expected(rows(root / 'data/accounts.csv'), rows(root / 'data/orders.csv'))
            # Literal independently calculated controls prevent a self-consistent oracle error.
            check('fixture-control', fixture['cohorts']['current'] == {'accounts': 40, 'converted_accounts': 14, 'conversion_rate': .35}
                  and fixture['cohorts']['pilot'] == {'accounts': 40, 'converted_accounts': 21, 'conversion_rate': .525}
                  and math.isclose(fixture['standardized']['pilot_rate'], .225)
                  and math.isclose(fixture['standardized']['difference_pp'], -12.5)
                  and fixture['excluded_orders'] == {'duplicate_deliveries': 3, 'unknown_accounts': 1, 'ineligible_accounts': 2, 'non_paid': 3},
                  'Fixed fixture matches independently counted account and order controls')
            actual = run(root / 'data', scratch / 'fixture.json')
            check('fixture-calculation', equivalent(actual, fixture), 'Counts, per-device rates, current-mix rates and percentage-point difference match')
            saved = json.loads((root / 'results.json').read_text(encoding='utf-8'))
            check('reproducible-output', equivalent(saved, fixture) and equivalent(saved, actual), 'Submitted results match fresh execution on immutable inputs')
        except Exception as exc:
            check('fixture-calculation', False, f'{type(exc).__name__}: {str(exc)[:120]}')
        for seed in (13, 71, 109):
            try:
                accounts, orders = generated(seed)
                data = scratch / f'data-{seed}'
                write_data(data, accounts, orders)
                actual = run(data, scratch / f'result-{seed}.json')
                check(f'alternate-{seed}', equivalent(actual, expected(accounts, orders)), 'Reordered, varied account populations and repeated deliveries calculate from inputs')
            except Exception as exc:
                check(f'alternate-{seed}', False, f'{type(exc).__name__}: {str(exc)[:100]}')
        try:
            accounts, orders = generated(202, missing_device=True)
            data = scratch / 'missing-device'
            write_data(data, accounts, orders)
            actual = run(data, scratch / 'missing-device.json')
            check('missing-stratum', equivalent(actual, expected(accounts, orders)), 'Unobserved pilot stratum remains null; standardized pilot rate and delta are null')
        except Exception as exc:
            check('missing-stratum', False, f'{type(exc).__name__}: {str(exc)[:100]}')
        try:
            accounts, orders = generated(303)
            orders.append(dict(orders[0], status='cancelled' if orders[0]['status'] == 'paid' else 'paid'))
            data = scratch / 'conflict'
            write_data(data, accounts, orders)
            output = scratch / 'conflict.json'
            result = subprocess.run([sys.executable, '-I', str(source), '--data', str(data), '--output', str(output)],
                                    cwd=root, capture_output=True, timeout=5)
            check('conflicting-delivery', result.returncode != 0 and not output.exists(), 'Conflicting order ID fails without emitting a successful result')
        except Exception as exc:
            check('conflicting-delivery', False, f'{type(exc).__name__}: {str(exc)[:100]}')
    brief = root / 'decision.md'
    check('brief-artifact', brief.is_file() and bool(brief.read_text(encoding='utf-8').strip()),
          'Decision brief exists; recommendation, provenance and causal reasoning require qualitative review')
    return checks


if __name__ == '__main__':
    try:
        result = main(Path(sys.argv[1]).resolve()) if len(sys.argv) == 2 else [{'id': 'usage', 'status': 'failed', 'detail': 'Expected workspace path'}]
    except Exception as exc:
        result = [{'id': 'verifier-error', 'status': 'failed', 'detail': f'{type(exc).__name__}: {str(exc)[:120]}'}]
    print(json.dumps({'checks': result}, separators=(',', ':')))
    sys.exit(0 if all(item['status'] == 'passed' for item in result) else 1)
