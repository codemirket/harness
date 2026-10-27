#!/usr/bin/env python3
"""Fetch and retain exact pinned skill-reference evidence.

This is an evidence collector, not a source installer. It stores immutable
retrieval metadata in planning/reviews and keeps raw reference/license bodies in
the task-private path recorded by each receipt (never in shipped source). A
failed request is retained as an unverified receipt and is never rewritten as
success by the catalog refresh command.
"""
from __future__ import annotations
import argparse, hashlib, json, os, re, sys
from datetime import datetime, timezone
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.parse import unquote
from urllib.request import Request, urlopen

ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/"planning/reviews/2026-10-27"
RAW=Path(os.environ.get("MIRKET_REFERENCE_RAW", "/tmp/mirket-reference-raw-2026-10-27"))
USER_AGENT="mirket-reference-audit/0.2"
LICENSE_OVERRIDES={
    "openai":["skills/.curated/define-goal/LICENSE.txt"],
    "anthropic":["skills/docx/LICENSE.txt"],
    "google":["LICENSE"],
    "cua":["LICENSE.md","apps/cua-spaces-macos/LICENSE"],
    "canvas-ui":["LICENSE.md"],
    "react-bits":["LICENSE.md"],
    "remotion":["LICENSE.md"],
    "anime":["LICENSE.md"],
    "motion":["LICENSE.md"],
    "motion-primitives":["LICENCE.md"],
    "statistical-analysis":[],
    "mengto-inspiration":["LICENSE"],
    "huashu-storyboard":["LICENSE"],
    "hyperframes-authoring":["LICENSE"],
}

def refs():
    rows=[]; seen=set(); tick=chr(96)
    for line in (ROOT/"planning/references.md").read_text().splitlines():
        p=line.split("|")
        if len(p)<3 or tick not in p[1]: continue
        ident=p[1].split(tick)[1]; urls=re.findall(r"https?://[^) ·]+",p[2])
        if urls and ident not in seen: seen.add(ident); rows.append((ident,urls))
    if not rows: raise SystemExit("reference plan has no selected references")
    return rows

def get(url):
    req=Request(url,headers={"User-Agent":USER_AGENT,"Accept":"text/plain,text/html,*/*"})
    try:
        with urlopen(req,timeout=30) as r:
            body=r.read(8*1024*1024)
            return {"status":int(r.status),"final_url":r.geturl(),"headers":{"content_type":r.headers.get("content-type"),"etag":r.headers.get("etag"),"last_modified":r.headers.get("last-modified")},"body":body,"error":None}
    except HTTPError as e:
        body=e.read(512*1024)
        return {"status":int(e.code),"final_url":e.geturl(),"headers":{"content_type":e.headers.get("content-type") if e.headers else None},"body":body,"error":f"HTTPError {e.code}"}
    except (URLError,TimeoutError,OSError) as e:
        return {"status":None,"final_url":url,"headers":{},"body":b"","error":f"{type(e).__name__}: {e}"}

def receipt(url, value, path):
    body=value["body"]; path.parent.mkdir(parents=True,exist_ok=True)
    if body: path.write_bytes(body)
    return {"requested_url":url,"final_url":value["final_url"],"status":value["status"],"headers":value["headers"],"bytes":len(body),"sha256":hashlib.sha256(body).hexdigest() if body else None,"raw_path":str(path) if body else None,"error":value["error"]}

def github_parts(url):
    m=re.match(r"https://github.com/([^/]+)/([^/]+)/(tree|blob)/([^/]+)(?:/(.*))?$",url)
    return m.groups() if m else None

def raw_url(owner,repo,rev,path):
    return f"https://raw.githubusercontent.com/{owner}/{repo}/{rev}/{path}"

def headings(body):
    text=body.decode("utf-8","replace")
    return [re.sub(r"^#+\s*","",x).strip() for x in text.splitlines() if re.match(r"^#{1,6}\s+",x)][:30]

def github_evidence(ident,url,page):
    parts=github_parts(url)
    if not parts: return {"source_file":None,"license_file":None,"sections":[],"receipts":[]}
    owner,repo,kind,rev,path=parts; receipts=[]; source=None; license=None
    if kind=="blob" and path:
        # A plan may point directly at a license (for example Geist).  Do not
        # mislabel that license as a technique-bearing source body.
        if re.search(r"(^|/)(license|licence|copying)(?:\.[^/]*)?$",path,re.I):
            license=path
        else:
            source=path
    else:
        html=page["body"].decode("utf-8","replace")
        links=[]
        for match in re.finditer(r'href="/'+re.escape(owner)+"/"+re.escape(repo)+r"/blob/"+re.escape(rev)+r'/([^"?#]+)',html):
            candidate=unquote(match.group(1))
            if candidate not in links: links.append(candidate)
        source=next((x for x in links if x.lower().endswith("/skill.md") or x.lower()=="skill.md"),None)
        source=source or next((x for x in links if x.lower().endswith("readme.md")),None)
        source=source or next((x for x in links if x.lower().endswith((".md",".mdx")) and "license" not in x.lower()),None)
        license=next((x for x in links if re.search(r"(^|/)(license|licence|copying)(\.[^/]*)?$",x,re.I)),None)
    licenses=LICENSE_OVERRIDES.get(ident)
    if licenses is None:
        licenses=[license] if license else []
    if ident in {"trail-of-bits","sentry-methods","qmd-methods"}: licenses=["LICENSE"]
    if source:
        source_url=raw_url(owner,repo,rev,source)
        v=get(source_url); receipts.append(receipt(source_url,v,RAW/f"{ident}-source-{hashlib.sha256(source_url.encode()).hexdigest()[:12]}.bin")); sections=headings(v["body"])
    else: sections=[]
    for number,license in enumerate(licenses):
        license_url=raw_url(owner,repo,rev,license)
        v=get(license_url); receipts.append(receipt(license_url,v,RAW/f"{ident}-license-{hashlib.sha256(license_url.encode()).hexdigest()[:12]}.bin"))
    return {"source_file":source,"license_file":licenses,"sections":sections,"receipts":receipts}

def collect(only=None):
    OUT.mkdir(parents=True,exist_ok=True); RAW.mkdir(parents=True,exist_ok=True)
    planned=dict(refs())
    selected_ids=set(only or planned)
    unknown=selected_ids-set(planned)
    if unknown: raise SystemExit(f"unknown reference IDs: {', '.join(sorted(unknown))}")
    existing={}
    fetch_path=OUT/"reference-fetches.json"
    if fetch_path.is_file():
        prior=json.loads(fetch_path.read_text())
        existing={record["id"]:record for record in prior.get("records",[]) if record.get("id") in planned}
    missing=set(planned)-selected_ids-set(existing)
    if missing:
        raise SystemExit("partial fetch requires retained receipts for: " + ", ".join(sorted(missing)))
    records=[]
    for ident,urls in planned.items():
        if ident not in selected_ids and ident in existing:
            records.append(existing[ident]); continue
        url_receipts=[]; selected={"source_file":None,"license_file":None,"sections":[],"receipts":[]}; selected_sources=[]; extra_urls=set()
        for index,url in enumerate(urls):
            v=get(url); page=receipt(url,v,RAW/f"{ident}-page-{index}.bin")
            url_receipts.append(page)
            if github_parts(url):
                evidence=github_evidence(ident,url,v)
                # Preserve every explicitly selected GitHub body/license, not
                # only the first URL in a multi-source family (notably Trail of
                # Bits and Sentry).  De-duplicate raw fetches by URL while
                # retaining the source candidate and its headings.
                new_receipts=[]
                for item in evidence["receipts"]:
                    if item["requested_url"] in extra_urls:
                        continue
                    extra_urls.add(item["requested_url"])
                    selected.setdefault("receipts",[]).append(item)
                    new_receipts.append(item)
                selected_sources.append({k:evidence[k] for k in ("source_file","license_file","sections")})
                selected_sources[-1]["receipts"]=new_receipts
                if selected["source_file"] is None and evidence.get("source_file"):
                    selected["source_file"]=evidence["source_file"]; selected["sections"]=evidence.get("sections",[])
                if selected["license_file"] is None and evidence.get("license_file"):
                    selected["license_file"]=evidence["license_file"]
        records.append({"id":ident,"urls":urls,"retrieved_at":datetime.now(timezone.utc).isoformat(),"url_receipts":url_receipts,"selected":selected,"selected_sources":selected_sources})
    obj={"schema":1,"generated_at":datetime.now(timezone.utc).isoformat(),"source_plan":"planning/references.md","source_count":len(records),"records":records}
    fetch_path.write_text(json.dumps(obj,indent=2,sort_keys=True)+"\n")
    print(json.dumps({"records":len(records),"failed":sum(1 for r in records for x in r["url_receipts"] if x["status"]!=200),"path":str(OUT/"reference-fetches.json")},indent=2))

if __name__=="__main__":
    p=argparse.ArgumentParser(); p.add_argument("--write",action="store_true"); p.add_argument("--only",action="append",default=[]); a=p.parse_args()
    if not a.write: raise SystemExit("pass --write to perform network fetches")
    collect(a.only or None)

