#!/usr/bin/env python3
"""Actual Pi TUI project init/status/reload against a fresh private installation.

Use explicit --node, --peers, and --state-root (an assembled disposable package's
parent). Evidence is retained in a new /private/tmp/mirket-pi-project-tui-* root.
This source-candidate fixture makes no model request or live installation change.
"""
import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import platform
import pty
import re
import selectors
import signal
import struct
import subprocess
import tempfile
import termios
import time
import traceback


parser = argparse.ArgumentParser(description=__doc__)
for name in ('node', 'peers', 'state-root'):
    parser.add_argument('--' + name, type=Path, required=True)
args = parser.parse_args()
assert all(path.is_absolute() for path in (args.node, args.peers, args.state_root))
assert str(args.state_root.resolve()).startswith('/private/tmp/mirket-')
packed = (args.state_root / 'package').resolve(strict=True)
node = args.node.resolve(strict=True)
peers = args.peers.resolve(strict=True)
cli = peers / '@earendil-works/pi-coding-agent/dist/cli.js'
repository = Path(__file__).resolve().parent.parent
root = Path(tempfile.mkdtemp(prefix='mirket-pi-project-tui-', dir='/private/tmp')).resolve()
os.chmod(root, 0o700)
print('Fixture:', root, flush=True)
home, agent, sessions, project = (root / name for name in ('home', 'agent', 'sessions', 'project'))
for directory in (home, agent, sessions, project):
    directory.mkdir(mode=0o700)
env = {key: value for key, value in os.environ.items()
       if not key.startswith(('PI_', 'MIRKET_', 'NODE_', 'OPENAI_', 'ANTHROPIC_'))}
env.update(HOME=str(home), PI_CODING_AGENT_DIR=str(agent),
           PI_CODING_AGENT_SESSION_DIR=str(sessions), PI_SKIP_UPDATE_CHECK='1',
           PI_TELEMETRY_DISABLED='1', TERM='xterm-256color', PATH='/nonexistent')
report = {'root': str(root), 'candidatePackage': str(packed), 'platform': platform.platform(),
          'architecture': platform.machine(), 'startedAt': time.time(), 'checks': [],
          'limits': ['Current first-party source candidate with minimal native core; '
                     'not released package or other-platform qualification.',
                     'Offline fixture provider; no paid model or image-delivery evidence.']}
transcript = bytearray()
child = master = slave = selector = None


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def check(label):
    report['checks'].append(label)
    (root / 'project-tui-report.json').write_text(json.dumps(report, indent=2) + '\n')
    print('PASS', label, flush=True)


def observe(seconds):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        if selector.select(min(.1, max(0, deadline - time.monotonic()))):
            try:
                chunk = os.read(master, 65536)
            except OSError:
                return
            if not chunk:
                return
            transcript.extend(chunk)
            assert len(transcript) <= 16 * 1024 * 1024, 'TUI transcript exceeds fixture budget'


def visible(start=0):
    text = bytes(transcript[start:]).decode('utf-8', errors='replace')
    text = re.sub(r'\x1b\][^\x07]*(?:\x07|\x1b\\)', '', text)
    return re.sub(r'\x1b\[[0-?]*[ -/]*[@-~]', '', text)


def send(value):
    os.write(master, value if isinstance(value, bytes) else value.encode() + b'\r')


def until(text, start=0, timeout=30):
    deadline = time.monotonic() + timeout
    while text not in visible(start):
        assert child.poll() is None, 'Pi exited before expected TUI state: ' + repr(text)
        assert time.monotonic() < deadline, 'Missing TUI state: ' + repr(text)
        observe(.1)


def trust_if_prompted(start=0):
    if 'Trust project folder?' in visible(start):
        # The admitted Pi host orders Trust / Trust parent / Trust this session.
        until('Trust (this session only)', start)
        send(b'\x1b[B\x1b[B\r')
        observe(2)
        report.setdefault('trustPrompts', []).append('Trust this disposable project for this session only')


def members():
    output = subprocess.check_output(['/bin/ps', '-axo', 'pid=,pgid=,stat=,command='], text=True)
    found = []
    for line in output.splitlines():
        match = re.match(r'\s*(\d+)\s+(\d+)\s+(\S+)\s+(.*)', line)
        if match and int(match[2]) == child.pid and not match[3].startswith('Z'):
            found.append({'pid': int(match[1]), 'command': match[4]})
    return found


try:
    report['node'] = subprocess.check_output([str(node), '--version'], env=env, text=True).strip()
    report['identity'] = {'nodePath': str(node), 'nodeSha256': sha(node), 'piEntry': str(cli),
                          'piVersion': json.loads((peers / '@earendil-works/pi-coding-agent/package.json').read_text())['version'],
                          'binarySha256': sha(packed / 'native/bin/mirket'),
                          'hostInventorySha256': sha(packed / 'host-inventory.json'),
                          'packageInventorySha256': sha(packed / 'package-inventory.json')}
    if platform.system() == 'Darwin':
        report['macOSVersion'] = subprocess.check_output(['/usr/bin/sw_vers', '-productVersion'], text=True).strip()
        report['macOSBuild'] = subprocess.check_output(['/usr/bin/sw_vers', '-buildVersion'], text=True).strip()
    setup = subprocess.run([str(node), str(packed / 'scripts/setup.mjs'), '--agent-dir', str(agent)],
                           cwd=repository, env=env, capture_output=True, text=True, timeout=660)
    (root / 'setup.stdout').write_text(setup.stdout)
    (root / 'setup.stderr').write_text(setup.stderr)
    assert setup.returncode == 0, 'Fresh private installation failed; inspect setup.stderr'
    report['releaseId'] = json.loads(setup.stdout)['releaseId']
    settings_file = agent / 'settings.json'
    settings = json.loads(settings_file.read_text())
    settings.update(enableInstallTelemetry=False, defaultProvider='offline-fixture',
                    defaultModel='deterministic', defaultTools=['+codemode', '+tool_search'],
                    lastChangelogVersion=report['identity']['piVersion'])
    settings_file.write_text(json.dumps(settings))
    global_bytes = settings_file.read_bytes()
    manual = '# Disposable TUI project\n\nPreserve these manual instructions exactly.\n'
    suffix = '\n## Manual instructions after Mirket\n\nPreserve this trailing content exactly.\n'
    guidance = project / 'AGENTS.md'
    guidance.write_text(manual)
    (project / '.pi').mkdir(mode=0o700)
    project_settings_file = project / '.pi/settings.json'
    unrelated = {'compaction': {'enabled': False}, 'projectFixture': 'preserve exactly', 'packages': []}
    project_settings_file.write_text(json.dumps(unrelated))
    provider = root / 'provider.mjs'
    provider.write_text('import * as ai from ' + json.dumps((peers / '@earendil-works/pi-ai/dist/index.js').as_uri())
                        + '; import {makeFixtureProvider} from '
                        + json.dumps((repository / 'tests/pi-offline-provider.mjs').as_uri())
                        + '; export default makeFixtureProvider(ai);\n')
    check('fresh private HOME, Pi agent, session directory and project; explicit private Node and Pi peer')
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack('HHHH', 60, 180, 0, 0))
    child = subprocess.Popen([str(node), str(cli), '--offline', '--provider', 'offline-fixture',
                              '--model', 'deterministic', '-e', str(provider)], cwd=project, env=env,
                             stdin=slave, stdout=slave, stderr=slave, start_new_session=True)
    os.close(slave)
    slave = None
    selector = selectors.DefaultSelector()
    selector.register(master, selectors.EVENT_READ)
    observe(4)
    trust_if_prompted()
    start = len(transcript)
    send('/mirket-project status')
    until('"status": "disabled"', start)
    check('actual TUI status visibly reports disabled before project opt-in')
    start = len(transcript)
    send('/mirket-project init library-cli +js-ts-navigation')
    until('"reloadRequired": true', start)
    until('"optedIn": true', start)
    catalog = json.loads((packed / 'skills/catalog.json').read_text())
    selected = sorted({skill['id'] for skill in catalog['global']}
                      | set(catalog['profiles']['library-cli']['skills']) | {'js-ts-navigation'})
    project_settings = json.loads(project_settings_file.read_text())
    assert {key: project_settings[key] for key in unrelated if key != 'packages'} == {
        key: value for key, value in unrelated.items() if key != 'packages'}
    package = [entry for entry in project_settings['packages'] if entry['source'] == str(packed)]
    assert len(package) == 1
    assert package[0]['skills'] == ['skills/' + skill + '/SKILL.md' for skill in selected]
    initialized = guidance.read_text()
    assert initialized.startswith(manual)
    assert initialized.count('<!-- mirket-project:start -->') == 1
    assert initialized.count('<!-- mirket-project:end -->') == 1
    metadata = re.search(r'<!-- mirket-project-data (\{[^\n]*\}) -->', initialized)
    assert metadata and json.loads(metadata[1]) == {'schema': 1, 'skills': [], 'tools': []}
    assert settings_file.read_bytes() == global_bytes, 'Project init changed global Pi settings'
    guidance.write_text(initialized + suffix)
    initialized = guidance.read_text()
    (root / 'AGENTS-initial.md').write_text(initialized)
    check('visible TUI init creates exact project skill filter and one managed block; preserves manual/global/unrelated settings')
    start = len(transcript)
    send('/reload')
    observe(3)
    trust_if_prompted(start)
    until('Reloaded keybindings, extensions, skills, prompts, themes, and context files', start, 45)
    check('literal TUI /reload completes with enrolled project settings and guidance')
    start = len(transcript)
    send('/mirket-project status')
    until('"status": "ready"', start)
    until('"guidanceFile":', start)
    assert guidance.read_text() == initialized
    assert json.loads(project_settings_file.read_text()) == project_settings
    check('actual TUI status visibly reports ready after reload without changing guidance or selection')
    start = len(transcript)
    send('/mirket-project init')
    until('"changed": false', start)
    until('"reloadRequired": true', start)
    assert guidance.read_text() == initialized, 'Repeated init changed manual content or managed block'
    assert json.loads(project_settings_file.read_text()) == project_settings
    assert settings_file.read_bytes() == global_bytes
    check('repeated TUI init preserves selected skills and exact manual prefix/trailing instructions')
    send(b'\x04')
    observe(2)
    child.wait(timeout=8)
    assert child.returncode == 0, 'Actual Pi TUI did not exit cleanly'
    report['exitCode'] = child.returncode
    check('actual Pi TUI exits cleanly with Ctrl-D')
    report['passed'] = True
except BaseException as error:
    report['failure'] = {'message': str(error), 'traceback': traceback.format_exc()}
    raise
finally:
    if child is not None:
        before = members()
        owned = all(member['pid'] == child.pid or str(root) in member['command']
                    or str(cli) in member['command'] or str(packed) in member['command'] for member in before)
        if before and owned:
            for sig in (signal.SIGTERM, signal.SIGKILL):
                if not members():
                    break
                try:
                    os.killpg(child.pid, sig)
                except ProcessLookupError:
                    break
                try:
                    child.wait(timeout=2)
                except subprocess.TimeoutExpired:
                    pass
        after = members()
        report['cleanup'] = {'before': before, 'after': after, 'refusedUnownedGroup': bool(before) and not owned}
        if after or not owned:
            report['passed'] = False
    if selector is not None:
        selector.close()
    for descriptor in (master, slave):
        if descriptor is not None:
            os.close(descriptor)
    report['finishedAt'] = time.time()
    (root / 'project-tui-transcript.txt').write_bytes(transcript)
    (root / 'project-tui-visible.txt').write_text(visible())
    (root / 'project-tui-report.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Evidence:', root / 'project-tui-report.json', flush=True)
    if child is not None:
        assert not after and owned, 'Disposable TUI process group survived or changed ownership'
print(json.dumps(report))
