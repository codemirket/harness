#!/usr/bin/env python3
"""Bind retained OCI export receipts to one compiled source for CI attestation."""
import json
from pathlib import Path
import sys
from image_audit import validate_export
root=Path(sys.argv[1]).resolve()
exports={alias:json.loads((root/(alias+'-export.json')).read_text()) for alias in ['design-tools','agent-runtime']}
sources={r['source_sha256'] for r in exports.values()}
if len(sources)!=1:raise ValueError('OCI candidates differ in compiled source')
source=sources.pop()
for alias,receipt in exports.items():validate_export(receipt,alias,source)
(root/'exports.json').write_text(json.dumps(exports,indent=2)+'\n')
predicate={'buildDefinition':{'buildType':'https://github.com/codemirket/harness/owned-oci/v1',
    'externalParameters':{'platform':'linux/arm64'},'resolvedDependencies':[{'uri':'mirket-source','digest':{'sha256':source}}]},
    'runDetails':{'builder':{'id':'https://github.com/codemirket/harness/.github/workflows/check.yml'},
        'byproducts':[{'name':name,'digest':{'sha256':r['archive_sha256']},'annotations':{'image_id':r['image_id']}} for name,r in exports.items()]}}
(root/'provenance.json').write_text(json.dumps(predicate,indent=2)+'\n')
