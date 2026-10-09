#!/usr/bin/env python3
"""Local HTTP SEO control exercise. Standard library; no search-engine contact."""
import argparse
import hashlib
from html.parser import HTMLParser
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import re
import threading
import urllib.request
from urllib.parse import urljoin, urlsplit
from datetime import datetime, timezone

ROBOTS = 'User-agent: *\nDisallow: /blocked/\nAllow: /blocked/public\n'
PATHS = ['/blocked/private', '/blocked/public', '/meta-noindex', '/header-noindex',
         '/canonical-conflict', '/preferred-a', '/preferred-b', '/clear-controls']


class Handler(BaseHTTPRequestHandler):
    remove_meta = False

    def do_GET(self):
        base = 'http://127.0.0.1:' + str(self.server.server_port)
        path = urlsplit(self.path).path
        headers = {'Content-Type': 'text/html; charset=utf-8'}
        if path == '/robots.txt':
            body = ROBOTS
            headers['Content-Type'] = 'text/plain'
        elif path == '/sitemap.xml':
            body = '<?xml version="1.0"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">' + ''.join('<url><loc>'+base+p+'</loc></url>' for p in ['/preferred-a', '/clear-controls']) + '</urlset>'
            headers['Content-Type'] = 'application/xml'
        elif path in PATHS:
            head = '<title>Fictional Fieldwork documentation</title>'
            if path == '/blocked/private' or (path == '/meta-noindex' and not self.remove_meta):
                head += '<meta name="robots" content="noindex, follow">'
            if path == '/header-noindex':
                headers['X-Robots-Tag'] = 'noindex'
            if path == '/canonical-conflict':
                head += '<link rel="canonical" href="'+base+'/preferred-a">'
                headers['Link'] = '<'+base+'/preferred-b>; rel="canonical"'
            else:
                head += '<link rel="canonical" href="'+base+path+'">'
            body = '<!doctype html><html lang="en"><head>'+head+'</head><body><h1>Fieldwork local fixture</h1><p>This is fictional documentation with readable server-rendered text.</p><nav>'+''.join('<a href="'+p+'">'+p+'</a> ' for p in PATHS)+'</nav></body></html>'
        else:
            self.send_error(404)
            return
        encoded = body.encode('utf-8')
        self.send_response(200)
        for key, value in headers.items():
            self.send_header(key, value)
        self.send_header('Content-Length', str(len(encoded)))
        self.end_headers()
        self.wfile.write(encoded)

    def log_message(self, *_):
        pass


class HeadSignals(HTMLParser):
    def __init__(self):
        super().__init__()
        self.in_head = False
        self.meta = []
        self.canonicals = []

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag == 'head':
            self.in_head = True
        if not self.in_head:
            return
        if tag == 'meta' and attrs.get('name', '').lower() in ('robots', 'googlebot'):
            self.meta += re.split(r'[\s,]+', attrs.get('content', '').lower())
        if tag == 'link' and 'canonical' in attrs.get('rel', '').lower().split():
            self.canonicals.append(attrs.get('href', ''))

    def handle_endtag(self, tag):
        if tag == 'head':
            self.in_head = False


def allowed_by_fixture_robots(text, path):
    # Deliberately scoped: one wildcard group, ASCII literal-prefix rules only.
    # Not a production robots.txt parser or an emulation of all Google rules.
    rules = []
    for line in text.splitlines():
        key, _, value = line.partition(':')
        value = value.strip()
        if key.lower() in ('allow', 'disallow') and value and path.startswith(value):
            rules.append((len(value), key.lower() == 'allow'))
    return max(rules, default=(0, True))[1]


def fetch(url, directory, label):
    # These requests are authorized local inspection, not a compliant crawl.
    request = urllib.request.Request(url, headers={'User-Agent': 'HarnessLocalSEOFixture/1.0'})
    # Ignore ambient proxy configuration: all exercise requests stay on loopback.
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
    with opener.open(request, timeout=5) as response:
        body = response.read(100000)
        record = {'url': url, 'status': response.status,
                  'headers': dict(response.headers.items()),
                  'retrieved_at': datetime.now(timezone.utc).isoformat(),
                  'body_sha256': hashlib.sha256(body).hexdigest()}
    (directory / (label+'.body')).write_bytes(body)
    (directory / (label+'.http.json')).write_text(json.dumps(record, indent=2)+'\n')
    return record, body.decode('utf-8')


def diagnose(record, body, robots):
    path = urlsplit(record['url']).path
    parser = HeadSignals()
    parser.feed(body)
    headers = {key.lower(): value for key, value in record['headers'].items()}
    directives = parser.meta + re.split(r'[\s,]+', headers.get('x-robots-tag', '').lower())
    header_canonical = re.findall(r'<([^>]+)>;\s*rel="canonical"', headers.get('link', ''))
    canonical_targets = {urljoin(record['url'], u) for u in parser.canonicals + header_canonical}
    allowed = allowed_by_fixture_robots(robots, path)
    noindex = bool({'noindex', 'none'} & set(directives))
    if not allowed:
        control = 'crawl_disallowed'
    elif noindex:
        control = 'noindex'
    elif len(canonical_targets) > 1:
        control = 'conflicting_canonical_signals'
    else:
        control = 'no_local_indexing_block_detected'
    return {'url': record['url'], 'http_status': record['status'],
            'robots_allowed': allowed, 'noindex_observed_by_local_inspector': noindex,
            'noindex_available_to_robots_compliant_crawler': noindex if allowed else None,
            'html_canonical': parser.canonicals, 'header_canonical': header_canonical,
            'local_control_diagnosis': control, 'google_indexing_status': 'unknown',
            'google_selected_canonical': 'unknown', 'search_console_access': 'unavailable'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    base = 'http://127.0.0.1:'+str(server.server_port)
    try:
        _, robots = fetch(base+'/robots.txt', args.output, 'robots')
        fetch(base+'/sitemap.xml', args.output, 'sitemap')
        findings = {}
        for path in PATHS:
            record, body = fetch(base+path, args.output, path.strip('/').replace('/', '-'))
            findings[path] = diagnose(record, body, robots)
        expected = {'/blocked/private': 'crawl_disallowed',
                    '/blocked/public': 'no_local_indexing_block_detected',
                    '/meta-noindex': 'noindex', '/header-noindex': 'noindex',
                    '/canonical-conflict': 'conflicting_canonical_signals',
                    '/preferred-a': 'no_local_indexing_block_detected',
                    '/preferred-b': 'no_local_indexing_block_detected',
                    '/clear-controls': 'no_local_indexing_block_detected'}
        checks = []
        for path, control in expected.items():
            assert findings[path]['local_control_diagnosis'] == control, path
            checks.append({'case': path, 'result': 'passed', 'expected': control})
        assert findings['/blocked/private']['noindex_available_to_robots_compliant_crawler'] is None
        checks.append({'case':'blocked-noindex-unobservable', 'result':'passed'})
        assert all(f['google_indexing_status'] == 'unknown' for f in findings.values())
        checks.append({'case':'actual-indexing-not-inferred', 'result':'passed'})
        # Mutation test: a naive report based on robots permission must fail.
        wrong = {p: ('no_local_indexing_block_detected' if f['robots_allowed'] else 'crawl_disallowed') for p, f in findings.items()}
        rejected = [p for p in expected if wrong[p] != expected[p]]
        assert {'/meta-noindex', '/header-noindex', '/canonical-conflict'} <= set(rejected)
        checks.append({'case':'wrong-robots-allowed-means-indexable', 'result':'rejected', 'mismatched_paths': rejected})
        # Change an actual response and ensure the sensor follows bytes, not URL labels.
        Handler.remove_meta = True
        record, body = fetch(base+'/meta-noindex', args.output, 'mutation-removed-meta')
        changed = diagnose(record, body, robots)
        assert changed['local_control_diagnosis'] == 'no_local_indexing_block_detected'
        checks.append({'case':'live-meta-removal-changes-diagnosis', 'result':'passed'})
        report = {'base_url':base, 'method':'bounded local HTTP fixture; no browser rendering or Google requests',
                  'findings':findings, 'checks':checks, 'mutation_finding':changed,
                  'source_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                  'limits':['Robots parser covers only this simple literal-prefix wildcard group.',
                            'Loopback is not publicly reachable; Google indexing is not observed.',
                            'No CDN, verified Googlebot IP, JS rendering or ranking outcome tested.']}
        (args.output/'report.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps({'output':str(args.output),'checks':len(checks),'status':'passed','bad_claim':'rejected'}))
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=2)
        Handler.remove_meta = False


if __name__ == '__main__':
    main()
