"""Target-specific paths and skill compatibility; capability sources stay portable."""
import json
import os
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CHOICES = ('codex', 'claude', 'all')


def load():
    data = json.loads((ROOT / 'registry/targets.json').read_text(encoding='utf-8'))
    if data.get('schema_version') != 1:
        raise ValueError('Unsupported target registry schema')
    return data['targets']


def normalize(name):
    if name not in ('codex', 'claude'):
        raise ValueError('Unsupported target: ' + str(name))
    return name


def names(choice):
    if choice == 'all':
        return ['codex', 'claude']
    return [normalize(choice)]


def skill_family(target):
    return normalize(target)


def project_skills_dir(target):
    return '.claude/skills' if skill_family(target) == 'claude' else '.agents/skills'


def global_adapter(target, home, adapter=None):
    target = normalize(target)
    return dict(adapter if adapter is not None else load()[target])


def validate_home_environment(home, selected):
    """Do not silently claim setup for a client using a nonstandard config root."""
    if home is not None:
        return
    expected = {'CODEX_HOME': Path.home() / '.codex'}
    relevant = []
    if 'codex' in selected:
        relevant.append('CODEX_HOME')
    if 'claude' in selected:
        relevant.append('CLAUDE_CONFIG_DIR')
        expected['CLAUDE_CONFIG_DIR'] = Path.home() / '.claude'
    for key in relevant:
        if os.environ.get(key) and Path(os.environ[key]).expanduser().resolve() != expected[key].resolve():
            raise ValueError(key + ' uses a custom config root; standard-home installation would target a different client. Use an isolated --home for export/rehearsal or standardize the client path first.')
