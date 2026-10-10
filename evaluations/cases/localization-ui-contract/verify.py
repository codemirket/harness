"""Fixture-specific protected-token checks; semantic translation needs review."""
import json
from pathlib import Path
import re
import sys


def substitutions(branch):
    """Count active # in this fixture's flat plural text, honoring ICU quoting."""
    quoted, count, index = False, 0, 0
    while index < len(branch):
        char = branch[index]
        if char == "'":
            if index + 1 < len(branch) and branch[index + 1] == "'":
                index += 2
                continue
            if quoted:
                quoted = False
            elif index + 1 < len(branch) and branch[index + 1] in '#{}':
                quoted = True
        elif char == '#' and not quoted:
            count += 1
        index += 1
    return count


def main(root):
    checks = []
    def check(name, passed, detail):
        checks.append(dict(id=name, status='passed' if passed else 'failed', detail=detail))
    source = json.loads((root/'en.json').read_text(encoding='utf-8'))
    target = json.loads((root/'tr.json').read_text(encoding='utf-8'))
    valid = isinstance(target, dict) and set(target) == set(source) and all(
        isinstance(v, str) and bool(v.strip()) for v in target.values())
    check('message-contract', valid, 'All source keys have nonempty target text')
    if not valid:
        return checks
    check('protected-placeholders', all(re.findall(r'\{(\w+)[},]', target[k]) ==
                                        re.findall(r'\{(\w+)[},]', source[k]) for k in source),
          'Application variables retain their names and occurrences')
    # Exact grammar for this public fixture only; this is not a general ICU parser.
    pattern = r'\{count,\s*plural,\s*=0\s*\{([^{}]+)\}\s*one\s*\{([^{}]+)\}\s*other\s*\{([^{}]+)\}\s*\}'
    plural = re.fullmatch(pattern, target['seats'])
    check('icu-schema', bool(plural) and [substitutions(branch) for branch in plural.groups()] == [0,1,1],
          'Three required flat branches retain active count substitutions; not a general ICU parser')
    check('markup', re.findall(r'</?[^>]+>', target['preview']) == ['<strong>', '</strong>']
          and '<strong>{name}</strong>' in target['preview'], 'The name retains its strong markup')
    check('amount-currency', [re.sub(r'\s+', '', n) for n in re.findall(r'[+\-−]?\s*\d+(?:[.,]\d+)*', target['price'])] == ['1.250,50']
          and bool(re.search(r'(?<![\d.,])(?:€\s*1\.250,50(?!\d|[.,]\d)|1\.250,50\s*(?:€|EUR\b))', target['price'])),
          'One exact EUR magnitude retained with Turkish decimal/grouping')
    deadline = target['deadline']
    check('deadline', sorted(re.findall(r'\d+', deadline)) == sorted(['17','00','10','2026'])
          and bool(re.search(r'(?<!\d)17:00(?!\d)', deadline))
          and bool(re.search(r'(?<![\w/])Europe/Istanbul(?![\w/])', deadline))
          and bool(re.search(r'(?<!\d)10\s+Ekim\s+2026(?!\d)', deadline)),
          'Exact date, time and IANA zone retained for this fixture')
    check('translated-values', all(target[k] != source[k] for k in source),
          'Values were revised; language and meaning still require qualitative review')
    qa = root/'translation-qa.md'
    check('qa-artifact', qa.is_file() and bool(qa.read_text(encoding='utf-8').strip()),
          'Review note exists; its claims must be inspected independently')
    return checks


if __name__ == '__main__':
    try:
        checks = main(Path(sys.argv[1]).resolve())
    except Exception as exc:
        checks = [dict(id='input-error', status='failed', detail=type(exc).__name__+': '+str(exc)[:160])]
    print(json.dumps({'checks': checks}))
    sys.exit(0 if all(c['status']=='passed' for c in checks) else 1)
