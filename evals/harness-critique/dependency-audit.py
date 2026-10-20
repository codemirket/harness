#!/usr/bin/env python3
"""Read-only public dependency/advisory queries; no installers or package imports."""
import concurrent.futures,json,re,tomllib,urllib.request,urllib.parse,time
from pathlib import Path
R=Path(__file__).resolve().parents[2]
OUT=R/'docs/harness-critique/dependency-audit.json'

def fetch(url,payload=None):
 req=urllib.request.Request(url,data=json.dumps(payload).encode() if payload else None,headers={'User-Agent':'Mirket-repository-review/1.0','Content-Type':'application/json'})
 with urllib.request.urlopen(req,timeout=30) as r: return json.load(r)

cargo=tomllib.loads((R/'Cargo.lock').read_text())['package']
manifest=tomllib.loads((R/'Cargo.toml').read_text())
node=json.loads((R/'runtime/design/tools/package-lock.json').read_text())['packages']
nm=json.loads((R/'runtime/design/tools/package.json').read_text())
packages=[]
for p in cargo:
 if p.get('source','').startswith('registry+'):packages.append(('crates.io',p['name'],p['version']))
for path,p in node.items():
 if path and p.get('version'):packages.append(('npm',p.get('name',path.split('node_modules/')[-1]),p['version']))
for line in (R/'runtime/agent/requirements.lock').read_text().splitlines():
 if ' @ ' not in line:continue
 name,url=line.split(' @ ',1);whl=url.split(' ')[0].split('/')[-1];version=whl.split('-')[1];packages.append(('PyPI',name,version))
packages=sorted(set(packages))
result={'date':'2026-10-20','scope':'Public locked package coordinates only; no project contents or credentials transmitted. OS packages, container images and complete upstream Penpot closure excluded.','osv':{'url':'https://api.osv.dev/v1/querybatch','packages':[],'errors':[]},'latest_direct':[]}
for i in range(0,len(packages),100):
 chunk=packages[i:i+100]
 try:
  reply=fetch(result['osv']['url'],{'queries':[{'package':{'ecosystem':e,'name':n},'version':v} for e,n,v in chunk]})
  for p,r in zip(chunk,reply['results']):result['osv']['packages'].append({'ecosystem':p[0],'name':p[1],'version':p[2],'result':r})
 except Exception as e:result['osv']['errors'].append({'offset':i,'count':len(chunk),'error':str(e)})
 print('OSV batch',i,flush=True)
latest=[]
for name in manifest['dependencies']:
 locked=[p['version'] for p in cargo if p['name']==name]
 latest.append(('crates.io',name,locked,f'https://crates.io/api/v1/crates/{name}'))
for name,v in nm['dependencies'].items():latest.append(('npm',name,[v],'https://registry.npmjs.org/'+urllib.parse.quote(name,safe='')))
for name,v in [('mem0ai','2.2.1'),('langgraph','1.2.16'),('langgraph-checkpoint-sqlite','3.1.2'),('fastembed','0.9.0'),('spacy','3.8.13')]:latest.append(('PyPI',name,[v],f'https://pypi.org/pypi/{name}/json'))
def query(p):
 eco,name,locked,url=p;r={'ecosystem':eco,'name':name,'locked':locked,'url':url}
 try:
  x=fetch(url)
  if eco=='crates.io':r['latest_stable']=x['crate'].get('max_stable_version');r['yanked_locked']=[v['num'] for v in x['versions'] if v['num'] in locked and v['yanked']]
  elif eco=='npm':r['latest_stable']=x['dist-tags'].get('latest');r['deprecated_locked']={v:x['versions'].get(v,{}).get('deprecated') for v in locked if x['versions'].get(v,{}).get('deprecated')}
  else:r['latest_stable']=x['info']['version'];r['yanked_locked']=[v for v in locked if x.get('releases',{}).get(v) and all(f.get('yanked') for f in x['releases'][v])]
 except Exception as e:r['error']=str(e)
 return r
with concurrent.futures.ThreadPoolExecutor(max_workers=5) as pool:result['latest_direct']=list(pool.map(query,latest))
result['osv']['requested_unique_packages']=len(packages)
result['advisory_details']=[]
for ident in sorted({v['id'] for p in result['osv']['packages'] for v in p['result'].get('vulns',[])}):
 try: result['advisory_details'].append(fetch('https://api.osv.dev/v1/vulns/'+ident))
 except Exception as error: result['osv']['errors'].append({'advisory':ident,'error':str(error)})
OUT.write_text(json.dumps(result,indent=2)+'\n')
print('AUDIT',len(packages),'queried',len(result['osv']['packages']),'errors',result['osv']['errors'])
for p in result['osv']['packages']:
 if p['result'].get('vulns'):print('ADVISORY',p['ecosystem'],p['name'],p['version'],p['result'])
for p in result['latest_direct']:
 if p.get('latest_stable') not in p['locked']:print('LATEST',p)
