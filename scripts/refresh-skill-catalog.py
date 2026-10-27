#!/usr/bin/env python3
"""Refresh skill hashes and build an audit from retained fetch receipts."""
from __future__ import annotations
import argparse, ast, hashlib, json, re
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
CATALOG=ROOT/"skills/catalog.json"
PROVENANCE=ROOT/"skills/PROVENANCE.md"
REVIEW_DIR=ROOT/"planning/reviews/2026-10-27"
FETCHES=REVIEW_DIR/"reference-fetches.json"
AUDIT=REVIEW_DIR/"reference-audit.json"
REVIEW=REVIEW_DIR/"skill-reference-audit.md"

# Explicit reviewed method adoption record. Refresh never infers source
# fidelity from a URL, body hash, or family name; the focused review names the
# reviewed sections and independently worded local technique.
METHOD_ADOPTIONS={
    "art-direction": {"families":["hallmark","taste"], "note":"Hallmark design-flow and Taste run/gate methods are explicitly mapped in source-design-adoption.md"},
    "design-critique": {"families":["hallmark","impeccable"], "note":"Hallmark review discipline and Impeccable anti-pattern review are explicitly mapped in source-design-adoption.md"},
    "interface-composition": {"families":["hallmark","taste"], "note":"Hallmark macrostructure and Taste brief-to-direction methods are explicitly mapped in source-design-adoption.md"},
    "motion-direction": {"families":["huashu-storyboard"], "note":"Huashu storyboard shot-card and energy/hold methods are explicitly mapped in source-design-adoption.md"},
    "scroll-motion": {"families":["emil"], "note":"Emil animation-decision methods are explicitly mapped for scroll context in source-design-adoption.md"},
    "ui-motion": {"families":["emil"], "note":"Emil animation-decision and review-format methods are explicitly mapped in source-design-adoption.md"},
    "vector-art": {"families":["mengto-inspiration"], "note":"MengTo pinned illustration-body construction methods are explicitly mapped in source-design-adoption.md"},
    "video-production": {"families":["hyperframes-authoring"], "note":"HyperFrames pinned producer readiness/asset and troubleshooting methods are explicitly mapped in source-design-adoption.md"},
}

# These are the only newly admitted families in the 2026-10-27 adoption pass.
# Their classification is source-level permission to adapt a reviewed method,
# not proof that any body faithfully reproduced the source.
NEW_SOURCE_METADATA={
    "huashu-storyboard": {
        "classification":"adaptable",
        "rights_status":"MIT at the pinned revision; independently worded method notes only; no Huashu runtime, renderer, assets or installer redistributed",
    },
    "hyperframes-authoring": {
        "classification":"adaptable",
        "rights_status":"Apache-2.0 at the pinned revision; independently worded authoring notes only; no HyperFrames runtime or package text redistributed",
    },
}
ADAPTED_FAMILIES={family for adoption in METHOD_ADOPTIONS.values() for family in adoption["families"]}
RESOURCE_COMPONENT=r"[a-z0-9](?:[a-z0-9_-]*[a-z0-9])?(?:\.[a-z0-9](?:[a-z0-9_-]*[a-z0-9])?)*"
RESOURCE=re.compile(r"references/(?:"+RESOURCE_COMPONENT+r"/)*"+RESOURCE_COMPONENT+r"\.md\Z")

def refs():
    out=[]; seen=set(); tick=chr(96)
    for line in (ROOT/"planning/references.md").read_text().splitlines():
        p=line.split("|")
        if len(p)<3 or tick not in p[1]: continue
        ident=p[1].split(tick)[1]; urls=re.findall(r"https?://[^) ·]+",p[2])
        if urls and ident not in seen: seen.add(ident); out.append({"id":ident,"urls":urls})
    if not out: raise ValueError("reference plan has no selected references")
    return out

def plan_memberships():
    tick=chr(96); count=0
    for line in (ROOT/"planning/references.md").read_text().splitlines():
        p=line.split("|")
        if len(p)>=3 and tick in p[1] and re.findall(r"https?://[^) ·]+",p[2]): count+=1
    return count

def frontmatter_description(body):
    """Read the bounded description field without importing a YAML parser."""
    match=re.search(r"^description:\s*(.+?)\s*$",body,re.M)
    if not match: return None
    raw=match.group(1)
    try:
        value=ast.literal_eval(raw)
    except (SyntaxError,ValueError):
        value=raw.strip().strip('"').strip("'")
    return value if isinstance(value,str) else None

def resource_records(skill):
    folder=ROOT/skill["path"].rsplit("/",1)[0]
    references=folder/"references"
    if references.is_symlink():
        raise ValueError(f"skill resource directory is a symlink: {references}")
    if not references.exists():
        # Extra files/directories are never silently ignored.
        extras=[p for p in folder.iterdir() if p.name!="SKILL.md"]
        if extras: raise ValueError(f"skill has unlisted resources: {extras[0]}")
        return []
    records=[]; actual_dirs=set()
    for path in sorted(folder.rglob("*")):
        if path.is_symlink():
            raise ValueError(f"skill resource is a symlink: {path}")
        relative=path.relative_to(folder).as_posix()
        if path.is_dir():
            if relative!="references" and not relative.startswith("references/"):
                raise ValueError(f"skill has unlisted resource directory: {path}")
            actual_dirs.add(relative)
            continue
        if not path.is_file():
            raise ValueError(f"skill resource is not a regular file: {path}")
        if relative=="SKILL.md": continue
        if not RESOURCE.fullmatch(relative):
            raise ValueError(f"skill resource path is outside the bounded reference contract: {path}")
        data=path.read_bytes()
        if len(data)>128*1024:
            raise ValueError(f"skill resource exceeds the bounded reference contract: {path}")
        records.append({"path":relative,"sha256":hashlib.sha256(data).hexdigest(),"bytes":len(data)})
    expected_dirs=set()
    for record in records:
        components=record["path"].split("/")[:-1]
        expected_dirs.update("/".join(components[:index]) for index in range(1,len(components)+1))
    if actual_dirs!=expected_dirs:
        raise ValueError(f"skill resource directories are empty or unlisted: {folder}")
    if len(records)>16: raise ValueError(f"skill has too many resources: {skill['id']}")
    return records

def adoption_record(skill):
    return METHOD_ADOPTIONS.get(skill["id"])

def update_provenance_records(provenance,catalog):
    """Keep the human-readable per-skill ledger aligned with the catalog."""
    for skill in catalog["global"]+catalog["project"]:
        adoption=adoption_record(skill)
        if not adoption: continue
        heading=f"### {skill['id']}"
        start=provenance.find(heading)
        if start<0: continue
        end=provenance.find("\n### ",start+len(heading))
        if end<0: end=len(provenance)
        section=provenance[start:end]
        section=re.sub(r"- Classification/terms: .*", "- Classification/terms: method-adapted; independently worded local method; source-level rights and notices are recorded in the focused adoption review.", section, count=1)
        section=re.sub(r"- (?:Comparison/link-only|Reviewed method) families: .*", "- Reviewed method families: "+", ".join(f"[{x}](#reference-{x})" for x in adoption["families"])+".", section, count=1)
        mapping=f"- Method mapping: {adoption['note']}; no upstream prose, code or assets copied."
        if "- Method mapping:" in section:
            section=re.sub(r"- Method mapping: .*", mapping, section, count=1)
        else:
            section += "\n"+mapping+"\n"
        provenance=provenance[:start]+section+provenance[end:]
    return provenance

def append_new_reference_records(provenance,plan_by_id):
    existing=provenance
    additions=[]
    for ident,meta in NEW_SOURCE_METADATA.items():
        if ident not in plan_by_id or f"id=\"reference-{ident}\"" in existing: continue
        additions.append(f'''<a id="reference-{ident}"></a>
### Reference: {ident}

- Classification: {meta["classification"]}; reviewed method family, not a redistributed body.
- Source selection: planning/references.md / focused design and video method adoption / {ident}.
'''+"\n".join(f"- Retained source {i}: {url}." for i,url in enumerate(plan_by_id[ident]["urls"],1))+f'''
- Terms/currentness: pinned retrieval and license receipts are retained in planning/reviews/2026-10-27/reference-audit.json; exact source/section rights govern future use.
- Removed assumptions: no upstream orchestration, runtime, package installation, account, renderer, remote fonts or branded assets.
''')
    if additions:
        marker="<!-- skill-reference-audit:begin -->"
        index=existing.find(marker)
        existing=existing[:index]+"\n".join(additions)+"\n\n"+existing[index:] if index>=0 else existing.rstrip()+"\n\n"+"\n".join(additions)+"\n"
    return "\n".join(line.rstrip() for line in existing.splitlines()) + "\n"

def update_reference_records(provenance):
    """Mark only explicitly mapped source families as adaptable in the ledger."""
    for ident in ADAPTED_FAMILIES:
        marker=f"<a id=\"reference-{ident}\"></a>"
        start=provenance.find(marker)
        if start<0: continue
        end=provenance.find("\n<a id=\"reference-",start+len(marker))
        if end<0: end=len(provenance)
        section=provenance[start:end]
        section=re.sub(r"- Classification: .*", "- Classification: adaptable; reviewed method family, no upstream body redistributed.", section, count=1)
        section=section.replace("retained comparison locator, no adapted section", "reviewed method locator; local mapping is independently worded")
        provenance=provenance[:start]+section+provenance[end:]
    return provenance

def body_contracts(catalog,source):
    out=[]
    for skill in catalog["global"]+catalog["project"]:
        if source not in skill.get("provenance",{}).get("reference_families",[]): continue
        body=(ROOT/skill["path"]).read_text()
        ps=[p.strip().replace("\n"," ") for p in re.split(r"\n\s*\n",body) if p.strip() and not p.startswith(("#","---","name:","description:"))]
        candidates=[p for p in ps if re.search(r"^(?:Deliver|Return|Output|Record|Verify|Before the final|Stop)\b",p,re.I)]
        ev=candidates[-1] if candidates else "Output/evidence mapping deferred to the focused source review; see the local skill body."
        adoption=adoption_record(skill)
        mapped=adoption and source in adoption["families"]
        out.append({"skill":skill["id"],"path":skill["path"],"skill_body_sha256":skill["source_sha256"],"trigger":skill["description"],"decision_method":ps[0] if ps else "See skill body","local_output_evidence":ev,"source_mapping":adoption["note"] if mapped else "not inferred by this generator; see focused source review documents"})
    return out

def make_audit(catalog):
    if not FETCHES.is_file(): raise ValueError("missing retained reference-fetches.json; run fetch-skill-reference-evidence.py --write")
    fetched=json.loads(FETCHES.read_text())
    fetched_records=fetched.get("records",[])
    if len(fetched_records)!=len({x.get("id") for x in fetched_records}): raise ValueError("duplicate fetch receipt IDs")
    rows={x["id"]:x for x in fetched_records}
    planned=refs()
    if set(rows)!={x["id"] for x in planned} or len(rows)!=len(planned): raise ValueError("fetch receipts do not exactly cover the approved reference plan")
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
            "reuse_boundary":"No upstream prose, code or assets were copied into shipped skill bodies; reviewed method adaptations are independently worded and classified in source-design-adoption.md."
        })
    return {"schema":2,"generated_at":fetched.get("generated_at"),"source_plan":"planning/references.md","source_count":len(planned),"plan_memberships":plan_memberships(),"shared_ids":["ponytail"],"body_review_documents":[str(p.relative_to(ROOT)) for p in review_docs],"sources":sources}

def refresh(write):
    catalog=json.loads(CATALOG.read_text()); changed=[]
    for skill in catalog["global"]+catalog["project"]:
        b=(ROOT/skill["path"]).read_bytes(); h=hashlib.sha256(b).hexdigest()
        if skill.get("source_sha256")!=h or skill.get("source_bytes")!=len(b):
            changed.append(skill["id"]); skill["source_sha256"],skill["source_bytes"]=h,len(b)
        description=frontmatter_description(b.decode("utf-8"))
        if description is not None and skill.get("description")!=description:
            changed.append("description:"+skill["id"]); skill["description"]=description
        resources=resource_records(skill)
        if skill.get("resources")!=resources:
            changed.append("resources:"+skill["id"]); skill["resources"]=resources
        adoption=adoption_record(skill)
        if adoption:
            provenance=skill.setdefault("provenance",{})
            if provenance.get("classification")!="method-adapted":
                provenance["classification"]="method-adapted"; changed.append("provenance:"+skill["id"])
            families=list(dict.fromkeys([*provenance.get("reference_families",[]),*adoption["families"]]))
            if families!=provenance.get("reference_families"):
                provenance["reference_families"]=families; changed.append("families:"+skill["id"])
            basis=provenance.setdefault("local_basis",[])
            adoption_basis="planning/reviews/2026-10-27/source-design-adoption.md / explicit reviewed method mapping"
            if adoption_basis not in basis:
                basis.append(adoption_basis); changed.append("basis:"+skill["id"])
            terms="repository LICENSE applies to independently worded local method; exact upstream terms/notices are retained per family in reference-audit receipts; no upstream prose/code/assets copied"
            if provenance.get("reuse_terms")!=terms:
                provenance["reuse_terms"]=terms; changed.append("terms:"+skill["id"])
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
    plans={x["id"]:x for x in refs()}
    by_id={x.get("id"):x for x in catalog.get("reference_sources",[])}
    for ident,plan in plans.items():
        if ident not in by_id:
            meta=NEW_SOURCE_METADATA.get(ident,{"classification":"comparison-only","rights_status":"rights and source body require exact review before adaptation"})
            entry={"id":ident,"urls":plan["urls"],"selection_locator":f"planning/references.md / selected source / {ident}","classification":meta["classification"],"inspection_status":"approved pinned locator; external body fetched in the 2026-10-27 focused adoption pass; source-to-skill fidelity is recorded only in focused review documents","rights_status":meta["rights_status"]}
            catalog.setdefault("reference_sources",[]).append(entry); by_id[ident]=entry; changed.append("reference:"+ident)
    for ref in catalog.get("reference_sources",[]):
        base=ref.get("inspection_status","").split("; 2026-10-27 retrieval receipts:",1)[0].rstrip()
        updated=base+inspection_suffix
        if ref.get("inspection_status")!=updated:
            ref["inspection_status"]=updated; changed.append("reference:"+ref.get("id",""))
    for ref in catalog.get("reference_sources",[]):
        if ref.get("id") in NEW_SOURCE_METADATA and ref.get("classification")!="adaptable":
            ref["classification"]="adaptable"; changed.append("reference-classification:"+ref["id"])
        if ref.get("id") in ADAPTED_FAMILIES and ref.get("classification")!="adaptable":
            ref["classification"]="adaptable"; changed.append("reference-classification:"+ref["id"])
        if ref.get("classification")=="adaptable" and ref.get("id") not in ADAPTED_FAMILIES:
            # A prior broad pass must not make historical comparison sources
            # look adopted. The original catalog classified all such families
            # comparison-only (the two link-only families remain untouched).
            ref["classification"]="comparison-only"; changed.append("reference-classification:"+ref["id"])
        if ref.get("id") not in ADAPTED_FAMILIES and ref.get("id") not in NEW_SOURCE_METADATA and ref.get("rights_status","").startswith("reviewed method locator"):
            ref["rights_status"]="no third-party text, code, assets, fonts or runtime redistributed; separate exact-section rights review required before adaptation"
            changed.append("reference-rights:"+ref["id"])
        if ref.get("id") in ADAPTED_FAMILIES:
            old=ref.get("inspection_status","")
            honest=old.replace("external body not fetched or adapted in this authoring pass", "external body reviewed in the focused adoption pass; only independently worded methods mapped")
            if honest!=old:
                ref["inspection_status"]=honest; changed.append("reference-review:"+ref["id"])
            if ref.get("id") not in NEW_SOURCE_METADATA:
                rights="reviewed method locator; exact license and notice receipts are retained in reference-audit.json; no upstream prose/code/assets redistributed"
                if ref.get("rights_status")!=rights:
                    ref["rights_status"]=rights; changed.append("reference-rights:"+ref["id"])
    catalog["reference_sources"]=sorted(catalog.get("reference_sources",[]),key=lambda x:x.get("id",""))
    obj=make_audit(catalog); encoded=json.dumps(obj,indent=2,sort_keys=True,ensure_ascii=False)+"\n"
    if not AUDIT.is_file() or AUDIT.read_text()!=encoded: changed.append("planning/reviews/2026-10-27/reference-audit.json")
    marker,end="<!-- skill-reference-audit:begin -->","<!-- skill-reference-audit:end -->"
    block=marker+"\nGenerated from retained reference-fetches.json on 2026-10-27. Retrieval status, final URL, headers, byte count and SHA-256 are preserved per approved URL; failed or missing source/license receipts remain unverified. Selected source file/section hashes are retained as source evidence; source-to-skill technique mappings are only those in the focused review documents and are not inferred by this generator. Detailed body analyses are retained in planning/reviews/2026-10-27/source-*.md. Raw bodies are retained only in the task-private path recorded by the receipts and are not shipped. No upstream prose, code or assets were copied into shipped skill bodies; the eight method-adapted mappings are explicit and independently worded.\nAudit artifact: planning/reviews/2026-10-27/reference-audit.json; fetch receipts: planning/reviews/2026-10-27/reference-fetches.json\n"+end
    provenance=PROVENANCE.read_text()
    provenance=provenance.replace("All 67 native skill bodies and their 335 public development cases are independently\nauthored for this repository. The approved local catalog and implementation\ncontracts supply the outcome/authority boundaries; no external prose, code,\nworkflow, asset or font has been copied or adapted into these bodies.","All 67 native skill bodies and their 335 public development cases are authored\nfor this repository. Eight project bodies are classified method-adapted because\nthey retain independently worded techniques from reviewed source families; no\nexternal prose, code, workflow, asset or font has been copied into these bodies.")
    provenance=provenance.replace("Every current body is classified independently-authored. The reference index below\npreserves all 58 retained families and URLs from the local plan solely for\ncomparison or link-only review.","Most bodies are independently-authored; eight project bodies are explicitly\nclassified method-adapted with reviewed, independently worded mappings. The\nreference index preserves all selected families and URLs for comparison or\nfocused method review.")
    provenance=update_provenance_records(provenance,catalog)
    provenance=append_new_reference_records(provenance,plans)
    provenance=update_reference_records(provenance)
    provenance=provenance.replace("skills/reference-audit.json","planning/reviews/2026-10-27/reference-audit.json")
    provenance=provenance.replace("Terms/currentness: not independently re-audited in this pass; exact source/section rights must be verified before adaptation.","Terms/currentness: reviewed from retained pinned-URL receipts on 2026-10-27; see planning/reviews/2026-10-27/reference-audit.json for license receipts and focused rights review. Exact source/section rights still govern any future adaptation.")
    provenance=provenance.replace("Terms/currentness: audited at the pinned URL on 2026-10-27; see planning/reviews/2026-10-27/reference-audit.json for the observed license and reuse classification. Exact source/section rights still govern any future adaptation.","Terms/currentness: reviewed from retained pinned-URL receipts on 2026-10-27; see planning/reviews/2026-10-27/reference-audit.json for license receipts and focused rights review. Exact source/section rights still govern any future adaptation.")
    provenance=re.sub(re.escape(marker)+r".*?"+re.escape(end),block,provenance,flags=re.S) if marker in provenance else provenance.rstrip()+"\n\n"+block+"\n"
    if PROVENANCE.read_text()!=provenance: changed.append("PROVENANCE.md")
    review="# Skill reference audit (2026-10-27)\n\n"+block.replace(marker+"\n","").replace("\n"+end,"")+f"\n\nThe audit contains {len(plans)} unique families and {plan_memberships()} plan memberships. It records exact URL receipts, pinned/moving status, selected raw source file and section headings, source/license body hashes, and local skill identities. It does not infer source-to-skill fidelity; that mapping is recorded only by the focused body-review documents. Any failed receipt remains unverified; an HTTP/tool receipt alone is not an operation or quality claim.\n"
    if not REVIEW.is_file() or REVIEW.read_text()!=review: changed.append("planning/reviews/2026-10-27/skill-reference-audit.md")
    if write:
        CATALOG.write_text(json.dumps(catalog,indent=2,ensure_ascii=False)+"\n"); AUDIT.write_text(encoded); PROVENANCE.write_text(provenance); REVIEW.write_text(review)
    return {"changed":changed,"references":len(plans),"audit_sha256":hashlib.sha256(encoded.encode()).hexdigest(),"write":write}

if __name__=="__main__":
    p=argparse.ArgumentParser(); p.add_argument("--write",action="store_true"); a=p.parse_args()
    result=refresh(a.write); print(json.dumps(result,indent=2))
    if result["changed"] and not a.write: raise SystemExit(1)

