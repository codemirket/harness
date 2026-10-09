"""Run concrete artifact checks with explicitly available local tools.

No dependency installation, model calls, account access or automatic acceptance.
"""
import argparse
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
KEYS = ('node', 'node_modules', 'python', 'browser', 'soffice', 'pdftoppm', 'pdfinfo')


def configuration_path():
    return Path(os.environ.get('HARNESS_TOOLCHAIN', str(Path.home() / '.agent-harness/toolchain.json'))).expanduser()


def load_tools():
    path = configuration_path()
    if path.is_symlink():
        raise ValueError('Toolchain configuration must not be a symlink')
    config = json.loads(path.read_text()) if path.exists() else {}
    if not isinstance(config, dict) or set(config) - set(KEYS) - {'schema_version'}:
        raise ValueError('Unknown toolchain configuration fields')
    if config and config.get('schema_version') != 1:
        raise ValueError('Unsupported toolchain schema')
    result = {}
    for key in KEYS:
        value = os.environ.get('HARNESS_' + key.upper(), config.get(key))
        if value is not None and (not isinstance(value, str) or not value):
            raise ValueError('Tool path must be nonempty text: ' + key)
        result[key] = value
    result['node'] = result['node'] or shutil.which('node')
    result['python'] = result['python'] or sys.executable
    result['soffice'] = result['soffice'] or shutil.which('soffice') or shutil.which('libreoffice')
    result['pdftoppm'] = result['pdftoppm'] or shutil.which('pdftoppm')
    result['pdfinfo'] = result['pdfinfo'] or shutil.which('pdfinfo')
    return result


def configure(values):
    path = configuration_path()
    for parent in [path] + list(path.parents):
        if parent.is_symlink():
            raise ValueError('Linked toolchain configuration path')
    existing = json.loads(path.read_text()) if path.exists() else {'schema_version': 1}
    if not isinstance(existing, dict) or set(existing) - set(KEYS) - {'schema_version'}:
        raise ValueError('Unknown existing toolchain fields')
    if existing.get('schema_version') != 1:
        raise ValueError('Unsupported toolchain schema')
    for key, value in values.items():
        if value is None:
            continue
        target = Path(value).expanduser().resolve()
        if key == 'node_modules':
            if not target.is_dir():
                raise ValueError('Missing module directory: ' + str(target))
        elif not target.is_file() or not os.access(target, os.X_OK):
            raise ValueError('Missing executable: ' + str(target))
        existing[key] = str(target)
    if not any(values.values()):
        raise ValueError('Specify at least one tool path; use doctor to inspect')
    path.parent.mkdir(parents=True, exist_ok=True)
    # Only tool paths, never tokens or account state, belong in this file.
    from .settings import _atomic_write
    _atomic_write(path, (json.dumps(existing, indent=2) + '\n').encode(), 0o600)
    return {'configuration': str(path), 'tools': existing, 'activation': 'paths_only'}


def environment(tools):
    env = dict(os.environ)
    for key, value in tools.items():
        if value:
            env['HARNESS_' + key.upper()] = value
    return env


def probe(argv, env):
    try:
        run = subprocess.run(argv, env=env, capture_output=True, text=True, timeout=15)
        return {'available': run.returncode == 0, 'detail': (run.stdout or run.stderr)[-1800:].strip()}
    except (OSError, subprocess.SubprocessError) as error:
        return {'available': False, 'detail': str(error)}


def doctor():
    tools = load_tools()
    env = environment(tools)
    checks = {}
    script = "const {createRequire}=require('node:module'); const path=require('node:path'); const r=createRequire(path.resolve(process.env.HARNESS_NODE_MODULES||process.cwd()+'/node_modules','__probe__.cjs')); const name=process.argv[1]; const p=r(name); console.log(name==='sharp'?p.versions.sharp:r(name+'/package.json').version)"
    for package in ('playwright', 'sharp'):
        checks[package] = probe([tools['node'], '-e', script, package], env) if tools['node'] else {'available': False, 'detail': 'Node.js is not configured'}
    for module in ('docx', 'pptx', 'openpyxl'):
        checks[module] = probe([tools['python'], '-c', 'import ' + module + '; print(' + module + '.__version__)'], env)
    for key in ('soffice', 'pdftoppm', 'pdfinfo'):
        checks[key] = probe([tools[key], '--version' if key == 'soffice' else '-v'], env) if tools[key] else {'available': False, 'detail': 'Not configured or on PATH'}
    return {'tools': tools, 'checks': checks,
            'available': {name: check['available'] for name, check in checks.items()},
            'limits': ['Import/version probes are prerequisites, not browser launch or document rendering proof.',
                       'Use browser scenarios and document demo/render on this host before claiming runtime readiness.',
                       'Raster image generation uses the host image tool or an explicitly connected provider; it is not supplied by these local checks.']}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    commands.add_parser('doctor', help='Probe existing local tools; never install dependencies')
    config = commands.add_parser('configure', help='Record explicit local executable/module paths')
    for key in KEYS:
        config.add_argument('--' + key.replace('_', '-'))
    for name in ('browser', 'documents', 'vector', 'markdown'):
        child = commands.add_parser(name, add_help=False, help='Use ' + name + ' --help for the artifact command')
        child.add_argument('args', nargs=argparse.REMAINDER)
    argv = list(sys.argv[1:] if argv is None else argv)
    if argv and argv[0] in ('browser', 'documents', 'vector', 'markdown'):
        args = argparse.Namespace(command=argv[0], args=argv[1:])
        extra = []
    else:
        args, extra = parser.parse_known_args(argv)
    if args.command == 'doctor':
        if extra:
            parser.error('Unexpected arguments')
        print(json.dumps(doctor(), indent=2))
        return 0
    if args.command == 'configure':
        if extra:
            parser.error('Unexpected arguments')
        print(json.dumps(configure({key: getattr(args, key) for key in KEYS}), indent=2))
        return 0
    # Keep each tool independently runnable; provider-specific SDKs are unnecessary.
    options = args.args + extra
    tools = load_tools()
    env = environment(tools)
    if args.command in ('browser', 'vector'):
        if not tools['node']:
            raise ValueError('Node.js is missing; configure an existing runtime with workbench configure')
        script = ROOT / 'scripts/workbench' / ('browser.mjs' if args.command == 'browser' else 'vector.mjs')
        command = [tools['node'], str(script), *options]
    else:
        script = ROOT / 'scripts/workbench' / ('documents.py' if args.command == 'documents' else 'markdown.py')
        command = [tools['python'] if args.command == 'documents' else sys.executable, str(script), *options]
    try:
        process = subprocess.Popen(command, env=env, start_new_session=os.name != 'nt')
        try:
            return process.wait(timeout=300)
        except (subprocess.TimeoutExpired, KeyboardInterrupt):
            if os.name != 'nt':
                os.killpg(process.pid, signal.SIGTERM)
            else:
                process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                if os.name != 'nt':
                    os.killpg(process.pid, signal.SIGKILL)
                else:
                    process.kill()
                process.wait()
            print('Workbench interrupted or exceeded 300 seconds; inspect partial output.', file=sys.stderr)
            return 1
    except OSError as error:
        raise ValueError('Cannot launch workbench: ' + str(error)) from error
