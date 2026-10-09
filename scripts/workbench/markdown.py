#!/usr/bin/env python3
"""Check local Markdown structure and destinations; never execute fenced examples.

This intentionally checks a documented Markdown subset, not prose quality or all
GitHub/CommonMark extensions. External links are recorded, never fetched.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import sys
from urllib.parse import unquote, urlsplit


def visible_lines(text, keep_code=False):
    lines, fence = [], None
    for number, line in enumerate(text.splitlines(), 1):
        match = re.match(r'^\s{0,3}(`{3,}|~{3,})', line)
        if match:
            token = match.group(1)
            if fence is None:
                fence = token
            elif token[0] == fence[0] and len(token) >= len(fence):
                fence = None
            continue
        if fence is None:
            lines.append((number, line.replace('`', '') if keep_code else re.sub(r'`[^`]*`', '', line)))
    return lines, fence


def headings(text):
    result, counts = [], {}
    for number, line in visible_lines(text, keep_code=True)[0]:
        match = re.match(r'^\s{0,3}(#{1,6})\s+(.+?)(?:\s+#+\s*)?$', line)
        if not match:
            continue
        title = match.group(2)
        slug = re.sub(r'[^\w\- ]', '', title.lower()).replace(' ', '-')
        occurrence = counts.get(slug, 0)
        counts[slug] = occurrence + 1
        result.append({'line': number, 'level': len(match.group(1)), 'title': title,
                       'anchor': slug + ('-' + str(occurrence) if occurrence else '')})
    return result


def inspect(source):
    source = source.resolve(strict=True)
    raw = source.read_bytes()
    if len(raw) > 2 * 1024 * 1024:
        raise ValueError('Markdown exceeds 2 MiB check limit')
    text = raw.decode('utf-8')
    lines, fence = visible_lines(text)
    issues, links, references = [], [], {}
    for number, line in lines:
        definition = re.match(r'^\s{0,3}\[([^]]+)\]:\s*(<[^>]+>|\S+)', line)
        if definition:
            references[definition.group(1).casefold()] = definition.group(2).strip('<>')
    if fence:
        issues.append({'kind': 'unclosed_fence', 'line': len(text.splitlines())})
    hs = headings(text)
    for previous, current in zip(hs, hs[1:]):
        if current['level'] > previous['level'] + 1:
            issues.append({'kind': 'heading_level_jump', 'line': current['line']})
    for number, line in lines:
        if re.match(r'^\s{0,3}\[[^]]+\]:', line):
            continue
        # Inline destinations including angle-wrapped paths; nested URL parentheses
        # are outside this lightweight check and must be reviewed in the renderer.
        destinations = [m.group(1) or m.group(2) for m in re.finditer(
            r'!?\[[^]]*\]\(\s*(?:<([^>]+)>|([^\s)]+))(?:\s+["\'][^\n]*?["\'])?\s*\)', line)]
        for match in re.finditer(r'!?\[([^]]+)\]\[([^]]*)\]', line):
            name = (match.group(2) or match.group(1)).casefold()
            if name not in references:
                issues.append({'kind': 'missing_reference', 'line': number, 'reference': name})
            else:
                destinations.append(references[name])
        for destination in destinations:
            parsed = urlsplit(destination)
            if parsed.scheme or parsed.netloc:
                links.append({'line': number, 'destination': destination, 'status': 'external_not_checked'})
                continue
            target = source.parent / unquote(parsed.path) if parsed.path else source
            entry = {'line': number, 'destination': destination, 'status': 'exists'}
            if not target.exists():
                entry['status'] = 'missing_file'
                issues.append({'kind': 'missing_file', 'line': number, 'destination': destination})
            elif parsed.fragment and target.suffix.lower() in ('.md', '.markdown'):
                anchor = unquote(parsed.fragment)
                if anchor not in {h['anchor'] for h in headings(target.read_text() if target.stat().st_size <= 2 * 1024 * 1024 else '')}:
                    entry['status'] = 'unresolved_anchor'
                    issues.append({'kind': 'unresolved_anchor', 'line': number, 'destination': destination})
            links.append(entry)
    return {'schema_version': 1, 'kind': 'markdown', 'input': str(source),
            'sha256': hashlib.sha256(raw).hexdigest(), 'headings': hs,
            'links': links, 'issues': issues, 'checks_passed': not issues,
            'review_required': ['Read the rendered document for hierarchy, density and meaning.',
                                'Verify technical claims and run relevant examples in their real environment.',
                                'Check external URLs and Markdown extensions with the project renderer.'],
            'limits': ['ATX headings, inline and full reference links only; explicit HTML anchors and nested destinations may need manual resolution.',
                       'No code was executed, external link fetched or writing-quality score assigned.']}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('input', type=Path)
    parser.add_argument('--output', type=Path, required=True, help='New report directory; existing content is never replaced')
    args = parser.parse_args(argv)
    sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
    from lib.bundle import safe_output
    output = safe_output(args.output)
    report = inspect(args.input)
    output.mkdir()
    (output / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))
    return 0 if report['checks_passed'] else 1


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except (OSError, ValueError) as error:
        print(str(error), file=sys.stderr)
        raise SystemExit(2)
