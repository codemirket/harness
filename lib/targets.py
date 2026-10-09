"""Target-specific paths and skill compatibility; capability sources stay portable."""
import json
import os
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
ALIASES = {'codex-desktop': 'codex', 'claude-code': 'claude', 'claude-desktop': 'claude'}
CHOICES = ('zed', 'codex', 'codex-desktop', 'claude-desktop', 'claude-code', 'claude', 'both', 'all')


def load():
    data = json.loads((ROOT / 'registry/targets.json').read_text(encoding='utf-8'))
    if data.get('schema_version') != 1:
        raise ValueError('Unsupported target registry schema')
    return data['targets']


def normalize(name):
    name = ALIASES.get(name, name)
    if name not in ('codex', 'zed', 'claude'):
        raise ValueError('Unsupported target: ' + str(name))
    return name


def names(choice):
    if choice == 'both':
        return ['codex', 'claude']
    if choice == 'all':
        return ['zed', 'codex', 'claude']
    return [normalize(choice)]


def skill_family(target):
    return 'claude' if normalize(target) == 'claude' else 'codex'


def project_skills_dir(target):
    return '.claude/skills' if skill_family(target) == 'claude' else '.agents/skills'


def global_adapter(target, home, adapter=None):
    target = normalize(target)
    result = dict(adapter if adapter is not None else load()[target])
    # Explicit --home is an isolation boundary. Never redirect it through the
    # calling user's APPDATA/XDG_CONFIG_HOME/CODEX_HOME environment.
    if target == 'zed' and sys.platform == 'win32':
        result['instruction_destination'] = 'AppData/Roaming/Zed/AGENTS.md'
        result['settings_destination'] = 'AppData/Roaming/Zed/settings.json'
    return result


def validate_home_environment(home, selected):
    """Do not silently claim setup for a client using a nonstandard config root."""
    if home is not None:
        return
    expected = {'CODEX_HOME': Path.home() / '.codex',
                'XDG_CONFIG_HOME': Path.home() / '.config',
                'APPDATA': Path.home() / 'AppData/Roaming'}
    relevant = []
    if 'codex' in selected:
        relevant.append('CODEX_HOME')
    if 'zed' in selected:
        relevant.append('APPDATA' if sys.platform == 'win32' else 'XDG_CONFIG_HOME')
    if 'claude' in selected:
        relevant.append('CLAUDE_CONFIG_DIR')
        expected['CLAUDE_CONFIG_DIR'] = Path.home() / '.claude'
    for key in relevant:
        if os.environ.get(key) and Path(os.environ[key]).expanduser().resolve() != expected[key].resolve():
            raise ValueError(key + ' uses a custom config root; standard-home installation would target a different client. Use an isolated --home for export/rehearsal or standardize the client path first.')
