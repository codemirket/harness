"""Regression tests for repository-owned skill hash/provenance refresh."""

import importlib.util
import json
import shutil
import tempfile
import unittest
from unittest.mock import patch
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/refresh-skill-catalog.py"
SPEC = importlib.util.spec_from_file_location("refresh_skill_catalog", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


class RefreshSkillCatalogTests(unittest.TestCase):
    def test_current_catalog_has_all_pinned_reference_contracts(self):
        result = MODULE.refresh(False)
        self.assertEqual(result["changed"], [])
        audit = json.loads((ROOT / "planning/reviews/2026-10-27/reference-audit.json").read_text())
        planned = MODULE.refs()
        self.assertEqual(audit["source_count"], len(planned))
        self.assertEqual(len(audit["sources"]), len(planned))
        self.assertEqual(audit["plan_memberships"], MODULE.plan_memberships())
        self.assertTrue(all(all(r["status"] == 200 for r in item["url_receipts"]) for item in audit["sources"]))
        skyvern = next(item for item in audit["sources"] if item["id"] == "skyvern")
        self.assertTrue(any("/LICENSE" in item["requested_url"] for item in skyvern["license_evidence"]))
        # Comparison-only families may intentionally have no local contract;
        # every newly adopted method family must have one.
        for ident in ("huashu-storyboard", "hyperframes-authoring"):
            source = next(item for item in audit["sources"] if item["id"] == ident)
            self.assertTrue(source["skill_contracts"])
            self.assertEqual(source["verification"], "verified-retrieval")
        self.assertTrue(audit["body_review_documents"])
        self.assertTrue(all(item["source_evidence"]["page_receipts"] for item in audit["sources"]))
        for item in audit["sources"]:
            for contract in item["skill_contracts"]:
                self.assertTrue(contract["path"])
                self.assertRegex(contract["skill_body_sha256"], r"^[0-9a-f]{64}$")
                self.assertTrue(contract["source_mapping"])
                self.assertNotIn("source_sections", contract)
                self.assertNotIn("source_body_sha256", contract)
        for family in ("trail-of-bits", "sentry-methods"):
            source = next(item for item in audit["sources"] if item["id"] == family)
            self.assertEqual(len(source["source_evidence"]["selected_sources"]), 2)

        catalog = json.loads((ROOT / "skills/catalog.json").read_text())
        methods = {x["id"]: x for x in [*catalog["global"], *catalog["project"]] if x["id"] in MODULE.METHOD_ADOPTIONS}
        self.assertEqual(set(methods), set(MODULE.METHOD_ADOPTIONS))
        self.assertTrue(all(x["provenance"]["classification"] == "method-adapted" for x in methods.values()))
        source_by_id = {x["id"]: x for x in catalog["reference_sources"]}
        self.assertTrue(all(source_by_id[x]["classification"] == "adaptable" for x in MODULE.ADAPTED_FAMILIES))
        for restricted in ("anthropic", "trail-of-bits"):
            self.assertNotIn(restricted, MODULE.ADAPTED_FAMILIES)
            self.assertIn(source_by_id[restricted]["classification"], {"link-only", "comparison-only"})
        self.assertNotIn("No upstream material was adapted into shipped skill bodies", (ROOT / "skills/PROVENANCE.md").read_text())
        self.assertTrue(all("No upstream material was adapted" not in item["reuse_boundary"] for item in audit["sources"]))

        for skill_id, expected in {
            "vector-art": "references/illustration-craft.md",
            "motion-direction": "references/storyboard-and-transitions.md",
            "video-production": "references/hyperframes-0.8.197-authoring.md",
        }.items():
            entry = next(x for x in catalog["project"] if x["id"] == skill_id)
            self.assertEqual([x["path"] for x in entry["resources"]], [expected])

    def test_fetch_collector_discovers_extension_license_in_pinned_tree(self):
        spec = importlib.util.spec_from_file_location(
            "fetch_skill_reference", ROOT / "scripts/fetch-skill-reference-evidence.py")
        collector = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(collector)
        pin = "a" * 40
        url = f"https://github.com/example/methods/tree/{pin}"
        page = {"body": (f'<a href="/example/methods/blob/{pin}/README.md">Method</a>'
                         f'<a href="/example/methods/blob/{pin}/LICENSE.md">Rights</a>').encode()}
        def response(requested):
            return {"status": 200, "final_url": requested, "headers": {},
                    "body": b"# Retained fixture", "error": None}
        with tempfile.TemporaryDirectory(prefix="mirket-license-receipt-") as name:
            with patch.object(collector, "RAW", Path(name)), patch.object(collector, "get", response):
                result = collector.github_evidence("fixture", url, page)
        self.assertEqual(result["license_file"], ["LICENSE.md"])
        self.assertEqual(len(result["receipts"]), 2)
        self.assertTrue(result["receipts"][1]["requested_url"].endswith("/LICENSE.md"))

    def test_fetch_collector_only_preserves_unselected_receipts(self):
        spec = importlib.util.spec_from_file_location(
            "fetch_skill_reference_partial", ROOT / "scripts/fetch-skill-reference-evidence.py")
        collector = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(collector)
        old_url = "https://example.invalid/old"
        new_url = "https://example.invalid/new"
        old = {"id": "old", "urls": [old_url], "retrieved_at": "original",
               "url_receipts": [{"requested_url": old_url, "status": 200}],
               "selected": {}, "selected_sources": []}
        with tempfile.TemporaryDirectory(prefix="mirket-partial-fetch-") as name:
            root = Path(name)
            (root / "reference-fetches.json").write_text(json.dumps({"records": [old]}))
            calls = []
            def response(requested):
                calls.append(requested)
                return {"status": 200, "final_url": requested, "headers": {},
                        "body": b"new body", "error": None}
            with patch.object(collector, "OUT", root), patch.object(collector, "RAW", root / "raw"), \
                 patch.object(collector, "refs", return_value=[("old", [old_url]), ("new", [new_url])]), \
                 patch.object(collector, "get", response):
                collector.collect(["new"])
            result = json.loads((root / "reference-fetches.json").read_text())
            self.assertEqual(calls, [new_url])
            self.assertEqual(next(x for x in result["records"] if x["id"] == "old")["retrieved_at"], "original")
            self.assertEqual(next(x for x in result["records"] if x["id"] == "new")["url_receipts"][0]["status"], 200)

    def test_drift_is_reported_and_write_refreshes_in_disposable_copy(self):
        with tempfile.TemporaryDirectory(prefix="mirket-skill-refresh-") as name:
            root = Path(name)
            shutil.copytree(ROOT / "skills", root / "skills")
            (root / "planning").mkdir()
            for pin in ("planning/references.md", "planning/skill-catalog-design.md", "planning/implementation-plan.md", "LICENSE", "planning/reviews/2026-10-27/reference-fetches.json"):
                destination = root / pin
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(ROOT / pin, destination)
            original = MODULE.ROOT
            original_paths = (MODULE.CATALOG, MODULE.PROVENANCE, MODULE.AUDIT, MODULE.REVIEW, MODULE.FETCHES, MODULE.REVIEW_DIR)
            try:
                MODULE.ROOT = root
                MODULE.CATALOG = root / "skills/catalog.json"
                MODULE.PROVENANCE = root / "skills/PROVENANCE.md"
                MODULE.REVIEW = root / "planning/reviews/2026-10-27/skill-reference-audit.md"
                MODULE.AUDIT = root / "planning/reviews/2026-10-27/reference-audit.json"
                MODULE.FETCHES = root / "planning/reviews/2026-10-27/reference-fetches.json"
                MODULE.REVIEW_DIR = root / "planning/reviews/2026-10-27"
                # File creation order must not change generated evidence order.
                for name in ("source-z.md", "source-a.md"):
                    (MODULE.REVIEW_DIR / name).write_text("Fixture review: `skyvern`\n")
                body = root / "skills/runtime-operations/SKILL.md"
                body.write_bytes(body.read_bytes() + b"\nControlled test drift.\n")
                self.assertIn("runtime-operations", MODULE.refresh(False)["changed"])
                MODULE.refresh(True)
                catalog = json.loads(MODULE.CATALOG.read_text())
                entry = next(x for x in catalog["global"] if x["id"] == "runtime-operations")
                self.assertEqual(entry["source_bytes"], body.stat().st_size)
                self.assertTrue(MODULE.AUDIT.exists())
                evidence = json.loads(MODULE.AUDIT.read_text())
                self.assertEqual(evidence["body_review_documents"], sorted(evidence["body_review_documents"]))
                skyvern = next(item for item in evidence["sources"] if item["id"] == "skyvern")
                self.assertEqual(skyvern["body_review_documents"], evidence["body_review_documents"])
                audit = MODULE.AUDIT
                audit.write_text(audit.read_text() + "\nmanual drift\n")
                self.assertIn("planning/reviews/2026-10-27/reference-audit.json", MODULE.refresh(False)["changed"])
                receipts = json.loads(MODULE.FETCHES.read_text())
                original_receipts = json.loads(json.dumps(receipts))
                receipts["records"][0]["url_receipts"][0]["requested_url"] = "https://forged.invalid/reference"
                MODULE.FETCHES.write_text(json.dumps(receipts))
                with self.assertRaisesRegex(ValueError, "requested URL"):
                    MODULE.refresh(False)
                receipts = json.loads(json.dumps(original_receipts))
                receipts["records"].append(json.loads(json.dumps(receipts["records"][0])))
                MODULE.FETCHES.write_text(json.dumps(receipts))
                with self.assertRaisesRegex(ValueError, "duplicate fetch receipt IDs"):
                    MODULE.refresh(False)
                receipts = json.loads(json.dumps(original_receipts))
                receipts["records"][0]["url_receipts"][0]["status"] = 503
                MODULE.FETCHES.write_text(json.dumps(receipts))
                MODULE.refresh(True)
                audited = json.loads(MODULE.AUDIT.read_text())
                first = next(item for item in audited["sources"] if item["id"] == receipts["records"][0]["id"])
                self.assertEqual(first["verification"], "unverified-retrieval")
                self.assertEqual(first["url_receipts"][0]["status"], 503)
                MODULE.AUDIT.unlink()
                MODULE.REVIEW.unlink()
                self.assertIn("planning/reviews/2026-10-27/reference-audit.json", MODULE.refresh(False)["changed"])
                self.assertIn("planning/reviews/2026-10-27/skill-reference-audit.md", MODULE.refresh(False)["changed"])
            finally:
                MODULE.ROOT = original
                MODULE.CATALOG, MODULE.PROVENANCE, MODULE.AUDIT, MODULE.REVIEW, MODULE.FETCHES, MODULE.REVIEW_DIR = original_paths

    def test_resource_drift_is_reported_and_rehashed(self):
        with tempfile.TemporaryDirectory(prefix="mirket-resource-refresh-") as name:
            root = Path(name)
            shutil.copytree(ROOT / "skills", root / "skills")
            (root / "planning").mkdir()
            for pin in ("planning/references.md", "planning/skill-catalog-design.md", "planning/implementation-plan.md", "LICENSE", "planning/reviews/2026-10-27/reference-fetches.json"):
                destination = root / pin
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(ROOT / pin, destination)
            original_paths = (MODULE.ROOT, MODULE.CATALOG, MODULE.PROVENANCE, MODULE.AUDIT, MODULE.REVIEW, MODULE.FETCHES, MODULE.REVIEW_DIR)
            try:
                MODULE.ROOT = root
                MODULE.CATALOG = root / "skills/catalog.json"
                MODULE.PROVENANCE = root / "skills/PROVENANCE.md"
                MODULE.REVIEW_DIR = root / "planning/reviews/2026-10-27"
                MODULE.AUDIT = MODULE.REVIEW_DIR / "reference-audit.json"
                MODULE.REVIEW = MODULE.REVIEW_DIR / "skill-reference-audit.md"
                MODULE.FETCHES = MODULE.REVIEW_DIR / "reference-fetches.json"
                resource = root / "skills/vector-art/references/illustration-craft.md"
                resource.write_text(resource.read_text() + "\nControlled resource drift.\n")
                self.assertIn("resources:vector-art", MODULE.refresh(False)["changed"])
                MODULE.refresh(True)
                catalog = json.loads(MODULE.CATALOG.read_text())
                entry = next(x for x in catalog["project"] if x["id"] == "vector-art")
                self.assertEqual(entry["resources"][0]["bytes"], resource.stat().st_size)
                self.assertEqual(entry["resources"][0]["sha256"], __import__("hashlib").sha256(resource.read_bytes()).hexdigest())
                self.assertEqual(MODULE.refresh(False)["changed"], [])
            finally:
                (MODULE.ROOT, MODULE.CATALOG, MODULE.PROVENANCE, MODULE.AUDIT, MODULE.REVIEW, MODULE.FETCHES, MODULE.REVIEW_DIR) = original_paths

    def test_empty_resource_directories_are_rejected(self):
        for nested in (False, True):
            with self.subTest(nested=nested), tempfile.TemporaryDirectory(prefix="mirket-empty-resource-") as name:
                root = Path(name)
                shutil.copytree(ROOT / "skills", root / "skills")
                (root / "planning").mkdir()
                for pin in ("planning/references.md", "planning/skill-catalog-design.md", "planning/implementation-plan.md", "LICENSE", "planning/reviews/2026-10-27/reference-fetches.json"):
                    destination = root / pin
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copyfile(ROOT / pin, destination)
                reference_dir = root / "skills/vector-art/references"
                if nested:
                    (reference_dir / "empty").mkdir()
                else:
                    for child in reference_dir.iterdir():
                        child.unlink()
                original_paths = (MODULE.ROOT, MODULE.CATALOG, MODULE.PROVENANCE, MODULE.AUDIT, MODULE.REVIEW, MODULE.FETCHES, MODULE.REVIEW_DIR)
                try:
                    MODULE.ROOT = root
                    MODULE.CATALOG = root / "skills/catalog.json"
                    MODULE.PROVENANCE = root / "skills/PROVENANCE.md"
                    MODULE.REVIEW_DIR = root / "planning/reviews/2026-10-27"
                    MODULE.AUDIT = MODULE.REVIEW_DIR / "reference-audit.json"
                    MODULE.REVIEW = MODULE.REVIEW_DIR / "skill-reference-audit.md"
                    MODULE.FETCHES = MODULE.REVIEW_DIR / "reference-fetches.json"
                    with self.assertRaisesRegex(ValueError, "empty or unlisted"):
                        MODULE.refresh(False)
                finally:
                    (MODULE.ROOT, MODULE.CATALOG, MODULE.PROVENANCE, MODULE.AUDIT, MODULE.REVIEW, MODULE.FETCHES, MODULE.REVIEW_DIR) = original_paths


if __name__ == "__main__":
    unittest.main()
