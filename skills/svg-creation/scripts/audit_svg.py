#!/usr/bin/env python3
"""Bounded structural inspection of standalone UTF-8 SVG; not a sanitizer."""
import argparse
from collections import Counter
import json
import math
from pathlib import Path
import re
import xml.etree.ElementTree as ET

SVG = 'http://www.w3.org/2000/svg'
MAX_BYTES = 2 * 1024 * 1024
NUMBER = r'[+-]?(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)(?:[eE][+-]?[0-9]+)?'
SEPARATOR = r'(?:[ \t\r\n]+,?[ \t\r\n]*|,[ \t\r\n]*)'
VIEWBOX = re.compile(r'[ \t\r\n]*' + NUMBER + '(?:' + SEPARATOR + NUMBER + '){3}[ \t\r\n]*')
CSS_TOKEN = re.compile(
    r'(?P<comment>/\*.*?\*/)|'
    r'(?P<url>\burl\(\s*(?:"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'|[^)"\']*)\s*\))|'
    r'(?P<string>"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\')', re.I | re.S)
RESOURCE_ATTRIBUTES = {'style', 'fill', 'stroke', 'filter', 'clip-path', 'mask',
                       'marker', 'marker-start', 'marker-mid', 'marker-end', 'cursor'}


def css_resources(text):
    """Inspect common literal URL tokens without treating comments/strings as URLs."""
    for token in CSS_TOKEN.finditer(text):
        if token.group('url'):
            value = token.group('url')[4:-1].strip()
            if value[:1] in ('"', "'"):
                value = value[1:-1]
            yield value.strip()


def css_import(text):
    plain = CSS_TOKEN.sub(lambda token: ' ' if token.group('comment') or token.group('string')
                         else token.group(0), text)
    return bool(re.search(r'@import\b', plain, re.I))


def audit(path):
    report = {'file': str(path), 'errors': [], 'warnings': [], 'elements': {},
              'ids': 0, 'local_references': 0}
    errors, warnings = report['errors'], report['warnings']
    try:
        with Path(path).open('rb') as stream:
            data = stream.read(MAX_BYTES + 1)
        if len(data) > MAX_BYTES:
            raise ValueError('Inspection limit is 2 MiB; inspect larger assets with project tools.')
        source = data.decode('utf-8-sig')
        if re.search(r'<!\s*(?:DOCTYPE|ENTITY)\b', source, re.I):
            raise ValueError('DTD/entity declarations are unsupported by this inspector.')
        root = ET.fromstring(source)
    except (OSError, UnicodeError, ET.ParseError, ValueError) as exc:
        # Parse diagnostics can contain markup; never echo supplied source text.
        errors.append(str(exc) if isinstance(exc, ValueError)
                      else 'Unable to read or parse UTF-8 XML (' + type(exc).__name__ + ').')
        return report
    if root.tag != '{' + SVG + '}svg':
        errors.append('Root must be svg in the SVG namespace.')
    raw_box = root.get('viewBox', '')
    if not VIEWBOX.fullmatch(raw_box):
        errors.append('viewBox must contain four finite SVG numbers.')
    else:
        box = [float(value) for value in re.findall(NUMBER, raw_box)]
        if not all(math.isfinite(value) for value in box) or box[2] <= 0 or box[3] <= 0:
            errors.append('viewBox must be finite with positive width and height.')

    ids, references, counts = set(), set(), Counter()
    external = active_handlers = False
    for element in root.iter():
        if not isinstance(element.tag, str):
            continue
        tag = element.tag.rsplit('}', 1)[-1]
        counts[tag] += 1
        identifier = element.get('id')
        if identifier is not None:
            if not identifier or any(char.isspace() for char in identifier):
                errors.append('An ID is empty or contains whitespace.')
            if identifier in ids:
                errors.append('Duplicate ID in asset.')
            ids.add(identifier)
        css_values = []
        if tag == 'style':
            css = ''.join(element.itertext())
            css_values.append(css)
            external |= css_import(css)
        for attribute, value in element.attrib.items():
            key = attribute.rsplit('}', 1)[-1]
            if key in RESOURCE_ATTRIBUTES:
                css_values.append(value)
            if key.lower().startswith('on'):
                active_handlers = True
            if key == 'href':
                if value.startswith('#'):
                    references.add(value[1:])
                elif value.strip():
                    external = True
            if key in ('aria-labelledby', 'aria-describedby'):
                references.update(value.split())
        for value in css_values:
            for target in css_resources(value):
                if target.startswith('#'):
                    references.add(target[1:])
                elif target:
                    external = True
    missing = references - ids
    if missing:
        errors.append(str(len(missing)) + ' unresolved local reference(s).')
    if counts['image']:
        warnings.append('image elements present: inspect for embedded/referenced raster content.')
    if counts['foreignObject']:
        warnings.append('foreignObject present: inspect export support and foreign content.')
    if counts['script'] or active_handlers:
        warnings.append('Active content present: inspect trust and embedding behavior.')
    if external or re.search(r'<\?xml-stylesheet\b', source, re.I):
        warnings.append('Nonlocal/data references or stylesheet dependencies present; inspect them.')
    if counts['text']:
        warnings.append('Live text present: verify font availability, shaping and export.')
    report.update(elements=dict(sorted(counts.items())), ids=len(ids),
                  local_references=len(references))
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('files', nargs='+', type=Path)
    parser.add_argument('--json', action='store_true')
    args = parser.parse_args()
    reports = [audit(path) for path in args.files]
    if args.json:
        print(json.dumps({'reports': reports, 'scope':
              'Structure only; no path/CSS conformance, sanitization or visual acceptance.'}, indent=2))
    else:
        for report in reports:
            print(report['file'] + ': ' + ('structural errors' if report['errors'] else 'structure checked'))
            for level in ('errors', 'warnings'):
                for message in report[level]:
                    print('  ' + level + ': ' + message)
        print('Inspect actual rendering, behavior and trust separately.')
    return 1 if any(report['errors'] for report in reports) else 0


if __name__ == '__main__':
    raise SystemExit(main())
