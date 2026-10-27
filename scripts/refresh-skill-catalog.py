#!/usr/bin/env python3
"""Refresh skill hashes and build an audit from retained fetch receipts."""
from __future__ import annotations
import argparse, hashlib, json, re
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
CATALOG=ROOT/"skills/catalog.json"
PROVENANCE=ROOT/"skills/PROVENANCE.md"
REVIEW_DIR=ROOT/"planning/reviews/2026-10-27"
FETCHES=REVIEW_DIR/"reference-fetches.json"
AUDIT=REVIEW_DIR/"reference-audit.json"
REVIEW=REVIEW_DIR/"skill-reference-audit.md"

def refs():
    out=[]; seen=set(); tick=chr(96)
    for line in (ROOT/"planning/references.md").read_text().splitlines():
        p=line.split("|")
        if len(p)<3 or tick not in p[1]: continue
        ident=p[1].split(tick)[1]; urls=re.findall(r"https?://[^) ·]+",p[2])
        if urls and ident not in seen: seen.add(ident); out.append({"id":ident,"urls":urls})
    if len(out)!=58: raise ValueError(f"expected 58 unique references, found {len(out)}")
    return out

def body_contracts(catalog,source):
    out=[]
    for skill in catalog["global"]+catalog["project"]:
        if source not in skill.get("provenance",{}).get("reference_families",[]): continue
        body=(ROOT/skill["path"]).read_text()
        ps=[p.strip().replace("\n"," ") for p in re.split(r"\n\s*\n",body) if p.strip() and not p.startswith(("#","---","name:","description:"))]
        candidates=[p for p in ps if re.search(r"^(?:Deliver|Return|Output|Record|Verify|Before the final|Stop)\b",p,re.I)]
        ev=candidates[-1] if candidates else "Output/evidence mapping deferred to the focused source review; see the local skill body."
        out.append({"skill":skill["id"],"path":skill["path"],"skill_body_sha256":skill["source_sha256"],"trigger":skill["description"],"decision_method":ps[0] if ps else "See skill body","local_output_evidence":ev,"source_mapping":"not inferred by this generator; see focused source review documents"})
    return out

def make_audit(catalog):
    if not FETCHES.is_file(): raise ValueError("missing retained reference-fetches.json; run fetch-skill-reference-evidence.py --write")
    fetched=json.loads(FETCHES.read_text())
    fetched_records=fetched.get("records",[])
    if len(fetched_records)!=len({x.get("id") for x in fetched_records}): raise ValueError("duplicate fetch receipt IDs")
    rows={x["id"]:x for x in fetched_records}
    planned=refs()
    if set(rows)!={x["id"] for x in planned} or len(rows)!=58: raise ValueError("fetch receipts do not exactly cover the approved 58 references")
    sources=[]
    review_docs=sorted(REVIEW_DIR.glob("source-*.md"))
    for plan in planned:
        rec=rows[plan["id"]]
        receipts=rec.get("url_receipts",[])
        if rec.get("urls") != plan["urls"]: raise ValueError(f"receipt URL list differs from approved plan: {plan['id']}")
        if len(receipts)!=len(plan["urls"]): raise ValueError(f"receipt count differs from approved plan: {plan['id']}")
        if [r.get("requested_url") for r in receipts] != plan["urls"]: raise ValueError(f"receipt requested URL differs from approved plan: {plan['id']}")
        if len({r.get("requested_url") for r in receipts}) != len(receipts): raise ValueError(f"duplicate receipt URL: {plan['id']}")
        statuses=[r.get("status") for r in receipts]
        selected=rec.get("selected",{})
        selected_receipts=selected.get("receipts",[])
        all_ok=bool(receipts) and all(s==200 for s in statuses) and all(r.get("status")==200 for r in selected_receipts)
        source_receipts=[r for r in selected_receipts if "LICENSE" not in r.get("requested_url","").upper() and "LICENCE" not in r.get("requested_url","").upper() and "COPYING" not in r.get("requested_url","").upper()]
        license_receipts=[r for r in selected_receipts if r not in source_receipts]
        section_data={"source_file":selected.get("source_file"),"sections":selected.get("sections",[]),"selected_sources":rec.get("selected_sources",[]),"body_receipts":source_receipts,"page_receipts":[{"requested_url":r.get("requested_url"),"status":r.get("status"),"bytes":r.get("bytes"),"sha256":r.get("sha256")} for r in receipts]}
        contracts=body_contracts(catalog,plan["id"])
        immutable_pin=all(re.match(r"https://github\.com/[^/]+/[^/]+/(?:tree|blob)/[0-9a-f]{40}(?:/|$)",u) for u in plan["urls"])
        sources.append({
            "id":plan["id"],"urls":plan["urls"],"retrieved_at":rec.get("retrieved_at"),
            "verification":"verified-retrieval" if all_ok else "unverified-retrieval",
            "pin_status":"immutable-git-pin" if immutable_pin else "moving-or-unpinned-url",
            "url_receipts":receipts,
            "source_evidence":section_data,
            "license_evidence":[{"requested_url":x.get("requested_url"),"status":x.get("status"),"sha256":x.get("sha256"),"bytes":x.get("bytes"),"raw_path":x.get("raw_path")} for x in license_receipts],
            "license_status":"license-receipt" if license_receipts and all(x.get("status")==200 for x in license_receipts) else "license-not-established",
            "skill_contracts":contracts,
            "body_review_documents":[str(p.relative_to(ROOT)) for p in review_docs if re.search(rf"`{re.escape(plan['id'])}`|\*\*{re.escape(plan['id'])}\*\*",p.read_text())],
            "reuse_boundary":"No upstream material was adapted into shipped skill bodies. Any future adaptation requires a separate rights review against these receipts."
        })
    return {"schema":2,"generated_at":fetched.get("generated_at"),"source_plan":"planning/references.md","source_count":58,"plan_memberships":59,"shared_ids":["ponytail"],"body_review_documents":[str(p.relative_to(ROOT)) for p in review_docs],"sources":sources}

def refresh(write):
    catalog=json.loads(CATALOG.read_text()); changed=[]
    for skill in catalog["global"]+catalog["project"]:
        b=(ROOT/skill["path"]).read_bytes(); h=hashlib.sha256(b).hexdigest()
        if skill.get("source_sha256")!=h or skill.get("source_bytes")!=len(b):
            changed.append(skill["id"]); skill["source_sha256"],skill["source_bytes"]=h,len(b)
    for pin in catalog["authoring_basis"]:
        h=hashlib.sha256((ROOT/pin["path"]).read_bytes()).hexdigest()
        if pin["sha256"]!=h: changed.append("basis:"+pin["path"]); pin["sha256"]=h
    # Keep the historical authoring-pass statement, but append the current
    # retrieval/review identity in the existing schema field.  This avoids a
    # second catalog schema while making the evidence discoverable per family.
    review_docs=sorted(REVIEW_DIR.glob("source-*.md"))
    review_identity="; ".join(f"{p.relative_to(ROOT)} sha256={hashlib.sha256(p.read_bytes()).hexdigest()}" for p in review_docs)
    inspection_suffix=("; 2026-10-27 retrieval receipts: planning/reviews/2026-10-27/reference-fetches.json; "
        f"focused source review evidence: {review_identity}; source-to-skill fidelity is recorded only in those reviews, not inferred by refresh")
    for ref in catalog.get("reference_sources",[]):
        base=ref.get("inspection_status","").split("; 2026-10-27 retrieval receipts:",1)[0].rstrip()
        updated=base+inspection_suffix
        if ref.get("inspection_status")!=updated:
            ref["inspection_status"]=updated; changed.append("reference:"+ref.get("id",""))
    obj=make_audit(catalog); encoded=json.dumps(obj,indent=2,sort_keys=True,ensure_ascii=False)+"\n"
    if not AUDIT.is_file() or AUDIT.read_text()!=encoded: changed.append("planning/reviews/2026-10-27/reference-audit.json")
    marker,end="<!-- skill-reference-audit:begin -->","<!-- skill-reference-audit:end -->"
    block=marker+"\nGenerated from retained reference-fetches.json on 2026-10-27. Retrieval status, final URL, headers, byte count and SHA-256 are preserved per approved URL; failed or missing source/license receipts remain unverified. Selected source file/section hashes are retained as source evidence; source-to-skill technique mappings are only those in the focused review documents and are not inferred by this generator. Detailed body analyses are retained in planning/reviews/2026-10-27/source-*.md. Raw bodies are retained only in the task-private path recorded by the receipts and are not shipped. No upstream material was adapted into shipped skill bodies.\nAudit artifact: planning/reviews/2026-10-27/reference-audit.json; fetch receipts: planning/reviews/2026-10-27/reference-fetches.json\n"+end
    provenance=PROVENANCE.read_text()
    provenance=provenance.replace("skills/reference-audit.json","planning/reviews/2026-10-27/reference-audit.json")
    provenance=provenance.replace("Terms/currentness: not independently re-audited in this pass; exact source/section rights must be verified before adaptation.","Terms/currentness: reviewed from retained pinned-URL receipts on 2026-10-27; see planning/reviews/2026-10-27/reference-audit.json for license receipts and focused rights review. Exact source/section rights still govern any future adaptation.")
    provenance=provenance.replace("Terms/currentness: audited at the pinned URL on 2026-10-27; see planning/reviews/2026-10-27/reference-audit.json for the observed license and reuse classification. Exact source/section rights still govern any future adaptation.","Terms/currentness: reviewed from retained pinned-URL receipts on 2026-10-27; see planning/reviews/2026-10-27/reference-audit.json for license receipts and focused rights review. Exact source/section rights still govern any future adaptation.")
    provenance=re.sub(re.escape(marker)+r".*?"+re.escape(end),block,provenance,flags=re.S) if marker in provenance else provenance.rstrip()+"\n\n"+block+"\n"
    if PROVENANCE.read_text()!=provenance: changed.append("PROVENANCE.md")
    review="# Skill reference audit (2026-10-27)\n\n"+block.replace(marker+"\n","").replace("\n"+end,"")+"\n\nThe audit contains 58 unique families and 59 plan memberships. It records exact URL receipts, pinned/moving status, selected raw source file and section headings, source/license body hashes, and local skill identities. It does not infer source-to-skill fidelity; that mapping is recorded only by the focused body-review documents. Any failed receipt remains unverified; an HTTP/tool receipt alone is not an operation or quality claim.\n"
    if not REVIEW.is_file() or REVIEW.read_text()!=review: changed.append("planning/reviews/2026-10-27/skill-reference-audit.md")
    if write:
        CATALOG.write_text(json.dumps(catalog,indent=2,ensure_ascii=False)+"\n"); AUDIT.write_text(encoded); PROVENANCE.write_text(provenance); REVIEW.write_text(review)
    return {"changed":changed,"references":58,"audit_sha256":hashlib.sha256(encoded.encode()).hexdigest(),"write":write}

if __name__=="__main__":
    p=argparse.ArgumentParser(); p.add_argument("--write",action="store_true"); a=p.parse_args()
    result=refresh(a.write); print(json.dumps(result,indent=2))
    if result["changed"] and not a.write: raise SystemExit(1)

