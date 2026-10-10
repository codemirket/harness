"""Check deliverable structure only; rendered meaning and craft require review."""
import json
import struct
import sys
import xml.etree.ElementTree as ET
from html.parser import HTMLParser
from pathlib import Path


class Page(HTMLParser):
    def __init__(self):
        super().__init__()
        self.assets = set()

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag in {'img', 'object'}:
            self.assets.add(attrs.get('src', attrs.get('data', '')))


def png_size(path):
    if not path.is_file():
        return None
    data = path.read_bytes()[:24]
    if len(data) != 24 or not data.startswith(b'\x89PNG\r\n\x1a\n'):
        return None
    return struct.unpack('>II', data[16:24])


def main(root):
    checks = []

    def check(identifier, condition, detail):
        checks.append({'id': identifier, 'status': 'passed' if condition else 'failed', 'detail': detail})

    page = Page()
    if (root / 'index.html').is_file():
        page.feed((root / 'index.html').read_text())
    check('integrated-assets', {'bird-art.png', 'workflow.svg'} <= page.assets,
          'Page embeds both deliverables; composition and actual loading require browser review')
    size = png_size(root / 'bird-art.png')
    check('raster-master', bool(size and min(size) >= 512),
          'PNG master supplied; content, anatomical craft and effective resolution require review')
    diagram = root / 'workflow.svg'
    labels, raster = set(), False
    if diagram.is_file():
        tree = ET.parse(diagram)
        for element in tree.iter():
            tag = element.tag.rsplit('}', 1)[-1]
            if tag == 'image':
                raster = True
            if tag == 'text':
                labels.add(' '.join(' '.join(element.itertext()).split()))
    facts = json.loads((root / 'facts.json').read_text())
    required = set(facts['nodes']) | {edge['label'] for edge in facts['edges']}
    check('native-diagram-labels', required <= labels and not raster,
          'Required labels are native SVG text, not embedded raster; rendered arrows, boundary and readability remain unverified')
    sizes = [png_size(root / 'evidence' / (name + '.png')) for name in ('desktop', 'narrow')]
    check('browser-captures', bool(all(sizes) and sizes[0][0] >= 900 and 300 <= sizes[1][0] <= 480 and all(s[1] >= 300 for s in sizes)),
          'Desktop/narrow PNG dimensions supplied; screenshot provenance, content and quality require actual inspection')
    note = root / 'evidence/inspection.md'
    check('inspection-record', note.is_file() and bool(note.read_text().strip()),
          'Inspection record exists; it is not independent acceptance')
    return checks


if __name__ == '__main__':
    try:
        result = main(Path(sys.argv[1]).resolve())
    except Exception as error:
        result = [{'id': 'verifier-error', 'status': 'failed', 'detail': type(error).__name__ + ': ' + str(error)[:120]}]
    print(json.dumps({'checks': result}))
    sys.exit(0 if all(item['status'] == 'passed' for item in result) else 1)
