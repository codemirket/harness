"""Package portable guidance for manual Claude Chat/Cowork account customization."""
import argparse
import json
from pathlib import Path
import shutil
import tempfile
import zipfile

from . import bundle, catalog, harness, target_install


def export(output):
    output = bundle.safe_output(output)
    config = harness.manifest()
    entries = {entry['id']: entry for entry in catalog.load_catalog()['skills']}
    with tempfile.TemporaryDirectory(prefix='.claude-handoff-', dir=output.parent) as temporary:
        stage = Path(temporary) / output.name
        stage.mkdir()
        (stage / 'instructions.txt').write_bytes((harness.ROOT / config.get('instructions', 'instructions/AGENTS.md')).read_bytes())
        included = []
        # The catalog's installer requires this host checkout and is deliberately
        # not presented as a usable account skill in a remote sandbox.
        for identifier in config['global_skills']:
            if identifier == 'skill-catalog':
                continue
            entry = entries[identifier]
            files = harness.tree_files(harness.ROOT / entry['path'])
            with zipfile.ZipFile(stage / (identifier + '.zip'), 'w', zipfile.ZIP_DEFLATED) as archive:
                for relative, raw in sorted(files.items()):
                    archive.writestr(entry['name'] + '/' + relative, raw)
            included.append(identifier)
        sources = target_install.mcp_sources()
        remote = {name: {'url': source['url'], 'description': source.get('description', '')}
                  for name, source in sources.items() if source['transport'] == 'http'}
        local = target_install.mcp_config('zed', {name: source for name, source in sources.items() if source['transport'] == 'stdio'})
        for name, value in [('remote-connectors.json', remote), ('chat-mcp-fragment.json', {'mcpServers': local})]:
            (stage / name).write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')
        (stage / 'README.txt').write_text(
            'Manual Claude Chat/Cowork handoff\n\n'
            'Upload selected ZIPs through Customize > Skills and enable them. Availability depends on the account.\n'
            'Use instructions.txt as reviewed account/project guidance where the UI supports it.\n'
            'Add remote-connectors.json URLs through account connectors. Complete authentication in the app.\n'
            'chat-mcp-fragment.json contains only local stdio servers. Review and merge it into the Chat desktop MCP config; do not replace the whole file.\n'
            'Cowork uses account Customize and connectors, not local Code configuration files.\n'
            'The local skill-catalog installer is omitted. References to host workflows/tools in other skills are conditional on availability.\n'
            'This export does not upload, activate, authenticate or verify any skill or connector.\n', encoding='utf-8')
        # Publish only to an absent path. safe_output rechecks linked parents.
        bundle.safe_output(output)
        shutil.copytree(stage, output)
    return {'output': str(output), 'skills': included, 'omitted': ['skill-catalog'], 'activation': 'manual_account_customize'}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args(argv)
    print(json.dumps(export(args.output), indent=2))
    return 0
