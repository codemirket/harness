#!/usr/bin/env python3
"""Produce reviewable pin proposals. Never edits sources or activates a runtime."""
import argparse
import datetime
import re
import hashlib
import json
from pathlib import Path
import urllib.request

ROOT = Path(__file__).resolve().parents[2]
OWNER = 'Mirket runtime maintainers'

def fetch_json(url):
    with urllib.request.urlopen(urllib.request.Request(url,headers={'User-Agent':'Mirket-maintenance/1'}),timeout=60) as r:
        return json.load(r)

def digest_archive(url):
    if not url.startswith('https://storage.googleapis.com/chrome-for-testing-public/'):
        raise ValueError('unexpected browser publisher')
    digest=hashlib.sha256();size=0
    with urllib.request.urlopen(url,timeout=120) as response:
        while chunk:=response.read(1024*1024):
            size+=len(chunk)
            if size>512*1024*1024:raise ValueError('browser archive exceeds review budget')
            digest.update(chunk)
    return {'sha256':digest.hexdigest(),'bytes':size}

def browser_proposal(current, stable, download=digest_archive):
    if current['version']==stable['version']:return None
    archives={}
    for arch,platform in [('arm64','linux-arm64'),('amd64','linux64')]:
        found=[x for x in stable['downloads']['chrome'] if x['platform']==platform]
        if len(found)!=1:raise ValueError('stable browser lacks a required architecture')
        url=found[0]['url']
        archives[arch]=current['archives'][arch]|{'url':url}|download(url)
    updated=current|{'version':stable['version'],'revision':stable['revision'],'archives':archives,
                     'provenance':current['provenance']|{'catalog_version':stable['version']}}
    return {'owner':OWNER,'file':'runtime/design/tools/browser.json','before':current,'after':updated,
            'dependent_files':['registry/design-stack.json','runtime/design/tools/Dockerfile'],
            'gates':['native check','sandboxed design-check','browser adversarial tests','artifact security gate'],
            'activation':'human review and explicit deployment authorization required'}

def node_proposal(current, releases, image_digest):
    candidates=[r for r in releases if r['version'].startswith('v24.') and r.get('lts')]
    if not candidates:raise ValueError('Node 24 LTS release inventory absent')
    selected=max(candidates,key=lambda r:tuple(map(int,r['version'][1:].split('.'))))
    version=selected['version'][1:];tag=version+'-trixie-slim'
    digest=image_digest(tag)
    reference='docker.io/library/node@'+digest
    if reference==current:return None
    return {'owner':OWNER,'file':'registry/design-stack.json','field':'images.node-build','before':current,'after':reference,
            'version':version,'publisher_tag':tag,'security_release':selected.get('security',False),
            'dependent_files':['runtime/design/tools/Dockerfile','runtime/design/kroki/Dockerfile','runtime/design/excalidraw/Dockerfile','.github/workflows/check.yml'],
            'gates':['native addons','design-check','three native CI targets','artifact security gate'],
            'activation':'human review and explicit deployment authorization required'}

def node_digest(tag):
    token=fetch_json('https://auth.docker.io/token?service=registry.docker.io&scope=repository:library/node:pull')['token']
    request=urllib.request.Request('https://registry-1.docker.io/v2/library/node/manifests/'+tag,headers={
        'Authorization':'Bearer '+token,'Accept':'application/vnd.oci.image.index.v1+json,application/vnd.docker.distribution.manifest.list.v2+json'})
    with urllib.request.urlopen(request,timeout=60) as r:
        body=r.read();reported=r.headers.get('Docker-Content-Digest')
    digest='sha256:'+hashlib.sha256(body).hexdigest()
    if reported!=digest:raise ValueError('registry manifest digest mismatch')
    platforms={m.get('platform',{}).get('architecture') for m in json.loads(body)['manifests']}
    if not {'arm64','amd64'}<=platforms:raise ValueError('Node image lacks a required architecture')
    return digest

def release_proposal(name, current, release, files):
    if release.get('draft') or release.get('prerelease'):
        raise ValueError('maintenance requires a stable published release')
    version=release['tag_name'].removeprefix('v')
    if version==current:return None
    assets=[{'name':a['name'],'url':a['browser_download_url'],'digest':a.get('digest')}
            for a in release.get('assets',[]) if a.get('digest','').startswith('sha256:')]
    if not assets:raise ValueError('native update lacks publisher artifact digests')
    return {'owner':OWNER,'component':name,'before':current,'after':version,
            'publisher':release['html_url'],'artifacts':assets,'dependent_files':files,
            'gates':['verify downloaded artifact digests/signatures','native ABI and semantic fixtures',
                     'vector parity and recall for embedding changes','artifact security gate'],
            'activation':'review exact architecture assets; no automatic activation'}

def snapshot_proposal(current, today):
    proposed=today.strftime('%Y%m%d')+'T000000Z'
    if proposed<=current:return None
    return {'owner':OWNER,'component':'Debian snapshot','before':current,'after':proposed,
            'publisher':'https://snapshot.debian.org/archive/debian/'+proposed+'/',
            'dependent_files':['runtime/design/tools/Dockerfile','runtime/design/kroki/Dockerfile','runtime/design/excalidraw/Dockerfile'],
            'gates':['signed APT Release verification','four owned image builds','ABI/browser/rendering tests','artifact security gate'],
            'activation':'proposal only; snapshot availability and signed metadata required before merge'}

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--output',type=Path,required=True);args=parser.parse_args()
    browser=json.loads((ROOT/'runtime/design/tools/browser.json').read_text())
    registry=json.loads((ROOT/'registry/design-stack.json').read_text())
    stable=fetch_json('https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions-with-downloads.json')['channels']['Stable']
    proposals=[browser_proposal(browser,stable),node_proposal(registry['images']['node-build'],fetch_json('https://nodejs.org/dist/index.json'),node_digest)]
    agent=json.loads((ROOT/'registry/agent-runtime.json').read_text())
    versions={t['id']:t['version'] for t in registry['tools']}
    for name,repo,version,files in [
        ('D2','d2lang/d2',versions['d2'],['runtime/design/tools/Dockerfile','runtime/design/tools/d2.go.mod','runtime/design/tools/d2.go.sum']),
        ('Typst','typst/typst',versions['typst'],['runtime/design/tools/Dockerfile','registry/design-stack.json']),
        ('ONNX','microsoft/onnxruntime',agent['versions']['onnxruntime'],['runtime/agent/native.json','registry/agent-runtime.json']),
    ]:
        proposals.append(release_proposal(name,version,fetch_json('https://api.github.com/repos/'+repo+'/releases/latest'),files))
    snapshots=re.findall(r'/archive/debian/(\d{8}T000000Z)/',(ROOT/'runtime/design/tools/Dockerfile').read_text())
    if not snapshots:raise ValueError('reviewed Debian snapshot missing')
    proposals.append(snapshot_proposal(max(snapshots),datetime.datetime.now(datetime.timezone.utc).date()))
    result={'schema':1,'owner':OWNER,'proposals':[p for p in proposals if p],
            'manual_review':['Distroless/Go image signature and lifecycle; use exact registry digest before merging','WASM/native codec package locks and upstream advisories','Kroki/Excalidraw parent locks and retirement of scoped overrides'],
            'cadence':{'advisory_review':'weekly','integrated_refresh':'monthly'},
            'response_targets':{'reachable_critical_triage':'1 business day','reachable_critical_containment':'72 hours','reachable_high_fix_or_mitigation':'7 days'},
            'limits':'Update proposals are not installation, compatibility verification, or automatic acceptance.'}
    with args.output.open('x') as f:json.dump(result,f,indent=2);f.write('\n')
    print(json.dumps({'proposals':len(result['proposals']),'output':str(args.output)}))
if __name__=='__main__':main()
