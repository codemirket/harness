"""Buildless website contracts; browser layout is checked separately."""
from html.parser import HTMLParser
from pathlib import Path
import unittest
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parents[1] / 'website'


class Page(HTMLParser):
    def __init__(self, source):
        super().__init__()
        self.ids = []
        self.links = []
        self.assets = []
        self.headings = 0
        self.scripts = 0
        self.inline = []
        self.feed(source)

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if 'id' in attrs:
            self.ids.append(attrs['id'])
        if tag == 'h1':
            self.headings += 1
        if tag == 'script':
            self.scripts += 1
        if 'style' in attrs or any(name.startswith('on') for name in attrs):
            self.inline.append(tag)
        if tag == 'a':
            self.links.append(attrs.get('href', ''))
        if tag in ('img', 'link'):
            url = attrs.get('src') if tag == 'img' else attrs.get('href')
            if tag == 'img' or attrs.get('rel') in ('stylesheet', 'icon'):
                self.assets.append(url)


class WebsiteTests(unittest.TestCase):
    def test_pages_are_local_csp_compatible_and_links_resolve(self):
        pages = {p.name: Page(p.read_text()) for p in ROOT.glob('*.html')}
        self.assertEqual(set(pages), {'index.html', 'legal.html', '404.html'})
        for name, page in pages.items():
            with self.subTest(page=name):
                self.assertEqual(page.headings, 1)
                self.assertEqual(len(page.ids), len(set(page.ids)))
                self.assertEqual(page.scripts, 0)
                self.assertEqual(page.inline, [])
                self.assertIn('main', page.ids)
                for asset in page.assets:
                    self.assertTrue(asset.startswith('/') and not asset.startswith('//'))
                    self.assertTrue((ROOT / asset.lstrip('/')).is_file(), asset)
                for link in page.links:
                    parsed = urlsplit(link)
                    if parsed.scheme:
                        self.assertEqual(parsed.scheme, 'https')
                        continue
                    self.assertFalse(parsed.netloc)
                    target = parsed.path.removeprefix('./').lstrip('/')
                    target = target or (name if not parsed.path else 'index.html')
                    self.assertIn(target, pages, link)
                    if parsed.fragment:
                        self.assertIn(parsed.fragment, pages[target].ids, link)

    def test_public_logo_alias_preserves_exact_original(self):
        self.assertEqual((ROOT / 'logo.png').read_bytes(), (ROOT / 'assets/mirket-logo.png').read_bytes())

    def test_parser_detects_broken_and_unsafe_fixture(self):
        page = Page('<h1>Test</h1><a href="#missing">Broken</a><script></script><p style="color:red">x</p>')
        self.assertNotIn('missing', page.ids)
        self.assertEqual(page.scripts, 1)
        self.assertEqual(page.inline, ['p'])


if __name__ == '__main__':
    unittest.main()
