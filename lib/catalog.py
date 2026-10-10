#!/usr/bin/env python3
"""Search the personal registry, plan profiles, and register reviewed skills. Python 3.9+."""
import argparse
from contextlib import contextmanager
import gzip
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import stat
import sys
import tarfile
import tempfile
import urllib.parse
import urllib.request
import unicodedata
import zlib

if __package__:
    from . import targets as target_registry
else:  # Preserve direct script and importlib-based review tooling.
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
    from lib import targets as target_registry

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / 'registry' / 'catalog.json'
INDEX = CATALOG.with_name('source-index.json')
RECEIPT = '.skill-catalog.json'
MAX_ARCHIVE = 100 * 1024 * 1024
MAX_EXPANDED_ARCHIVE = 512 * 1024 * 1024
MAX_ARCHIVE_MEMBERS = 100000
MAX_ARCHIVE_METADATA_DEPTH = 32
MAX_PAYLOAD = 32 * 1024 * 1024
MAX_TREE = 16 * 1024 * 1024
MAX_FILES = 4096


def is_link(path):
    """Include Windows junctions/reparse points on Python 3.9+."""
    try:
        info = path.lstat()
    except FileNotFoundError:
        return False
    return stat.S_ISLNK(info.st_mode) or bool(
        getattr(info, 'st_file_attributes', 0)
        & getattr(stat, 'FILE_ATTRIBUTE_REPARSE_POINT', 0x400))


def safe_path(value):
    path = PurePosixPath(value)
    if (not value or '\\' in value or ':' in value or path.is_absolute()
            or any(part in ('', '.', '..') for part in value.split('/'))):
        raise ValueError('Unsafe catalog/archive path: ' + value)
    reserved = {'CON', 'PRN', 'AUX', 'NUL', 'CONIN$', 'CONOUT$'}
    reserved.update(prefix + str(n) for prefix in ('COM', 'LPT') for n in range(1, 10))
    for part in path.parts:
        if (part.endswith(('.', ' ')) or part.split('.')[0].upper() in reserved
                or any(ord(char) < 32 or char in '<>\"|?*' for char in part)):
            raise ValueError('Non-portable catalog/archive path: ' + value)
    return path


def validate_paths(files):
    portable_paths = {}
    file_paths = set(files)
    for relative in files:
        parts = safe_path(relative).parts
        for length in range(1, len(parts)):
            if '/'.join(parts[:length]) in file_paths:
                raise ValueError('File/directory collision in payload: ' + relative)
        for length in range(1, len(parts) + 1):
            prefix = '/'.join(parts[:length])
            key = unicodedata.normalize('NFC', prefix).casefold()
            if key in portable_paths and portable_paths[key] != prefix:
                raise ValueError('Case-colliding payload paths: ' + prefix)
            portable_paths[key] = prefix


def load_catalog(path=CATALOG):
    data = json.loads(path.read_text(encoding='utf-8'))
    if data.get('schema_version') != 1:
        raise ValueError('Unsupported catalog schema')
    identifiers = [entry['id'] for entry in data['skills']]
    if len(identifiers) != len(set(identifiers)):
        raise ValueError('Duplicate catalog identifiers')
    return data


def payload_hash(files):
    digest = hashlib.sha256()
    for name in sorted(files):
        digest.update(name.encode('utf-8') + b'\0')
        digest.update(hashlib.sha256(files[name]).digest())
    return digest.hexdigest()


def selection_files(entry):
    """Validate explicit source-to-payload mappings before reading any source."""
    safe_path(entry['path'])
    extras = entry.get('extra_files', {})
    if not isinstance(extras, dict):
        raise ValueError('extra_files must be a source-to-destination mapping')
    for source, target in extras.items():
        if not isinstance(source, str) or not isinstance(target, str):
            raise ValueError('extra_files paths must be strings')
        safe_path(source)
        safe_path(target)
    for name in entry['license_files']:
        safe_path(name)
    targets = list(extras.values())
    if len(targets) != len(set(targets)):
        raise ValueError('Duplicate extra file destination')
    if RECEIPT in targets:
        raise ValueError('Extra file conflicts with receipt name')
    validate_paths(targets + [RECEIPT])
    return extras


def member_targets(relative, entry):
    targets = []
    prefix = entry['path'].rstrip('/') + '/'
    if relative.startswith(prefix):
        targets.append(str(safe_path(relative[len(prefix):])))
    if relative in entry.get('extra_files', {}):
        targets.append(str(safe_path(entry['extra_files'][relative])))
    if relative in entry['license_files']:
        targets.append('.upstream-licenses/' + str(safe_path(relative)))
    if len(targets) != len(set(targets)):
        raise ValueError('Duplicate selected destination for: ' + relative)
    return targets


def select_member(relative, entry):
    targets = member_targets(relative, entry)
    return targets[0] if targets else None


def check_selected(files, entry):
    for name in entry['license_files']:
        if '.upstream-licenses/' + name not in files:
            raise ValueError('Missing selected license: ' + name)
    for source, target in entry.get('extra_files', {}).items():
        if target not in files:
            raise ValueError('Missing selected extra file: ' + source)
    if len(files) > MAX_FILES:
        raise ValueError('Skill payload exceeds file count limit')
    if RECEIPT in files:
        raise ValueError('Upstream payload conflicts with receipt name')
    validate_paths(list(files) + [RECEIPT])


def add_selected(selected, target, value):
    if target in selected:
        raise ValueError('Duplicate selected destination: ' + target)
    selected[target] = value


def local_regular(source, relative):
    path = source
    for part in safe_path(relative).parts:
        path = path / part
        if is_link(path):
            raise ValueError('Symlink in selected source path: ' + relative)
    if not path.is_file() or source not in path.resolve().parents:
        raise ValueError('Missing or unsafe selected source file: ' + relative)
    return path


def read_local(source, entry):
    """Read a reviewed checkout without following file/directory symlinks."""
    selection_files(entry)
    source = source.resolve()
    selected = {}
    skill_dir = source
    for part in safe_path(entry['path']).parts:
        skill_dir = skill_dir / part
        if is_link(skill_dir):
            raise ValueError('Missing or symlinked skill directory')
    if not skill_dir.is_dir() or source not in skill_dir.resolve().parents:
        raise ValueError('Missing or symlinked skill directory')
    relatives = set(entry['license_files']) | set(entry.get('extra_files', {}))
    for root, directories, names in os.walk(skill_dir, followlinks=False):
        for name in directories + names:
            if is_link(Path(root) / name):
                raise ValueError('Symlink in skill payload: ' + name)
        for name in names:
            relatives.add((Path(root) / name).relative_to(source).as_posix())
    for relative in sorted(relatives):
        path = local_regular(source, relative)
        for target in member_targets(relative, entry):
            add_selected(selected, target, path)
    check_selected(selected, entry)
    if sum(path.stat().st_size for path in selected.values()) > MAX_PAYLOAD:
        raise ValueError('Skill payload exceeds size limit')
    files = {}
    total = 0
    for target, path in selected.items():
        with path.open('rb') as stream:
            content = stream.read(MAX_PAYLOAD - total + 1)
        total += len(content)
        if total > MAX_PAYLOAD:
            raise ValueError('Skill payload exceeds size limit')
        files[target] = content
    return files


class _ExpandedArchiveReader:
    """Bound gzip output before tar parsing, including metadata and skipped files."""

    def __init__(self, stream):
        self.stream = stream
        self.remaining = MAX_EXPANDED_ARCHIVE

    def read(self, size=-1):
        size = self.remaining + 1 if size < 0 else min(size, self.remaining + 1)
        content = self.stream.read(size)
        self.remaining -= len(content)
        if self.remaining < 0:
            raise ValueError('Source archive exceeds expanded size limit')
        return content


class _SourceTarInfo(tarfile.TarInfo):
    """Strict GitHub tar framing with budgets before extended-header processing."""

    @classmethod
    def fromtarfile(cls, archive):
        depth = getattr(archive, '_source_metadata_depth', 0) + 1
        if depth > MAX_ARCHIVE_METADATA_DEPTH:
            raise ValueError('Source archive exceeds metadata nesting limit')
        archive._source_metadata_depth = depth
        try:
            return super().fromtarfile(archive)
        except tarfile.EOFHeaderError:
            if depth != 1:
                raise ValueError('Source archive metadata has no following member')
            if archive.fileobj.read(tarfile.BLOCKSIZE) != b'\0' * tarfile.BLOCKSIZE:
                raise ValueError('Source archive requires two zero end blocks')
            archive._source_end_seen = True
            raise
        except tarfile.HeaderError as error:
            # TarFile.next otherwise accepts invalid/truncated non-first headers
            # as EOF, even with errorlevel=2.
            raise ValueError('Invalid or truncated tar source archive') from error
        finally:
            archive._source_metadata_depth = depth - 1

    def _proc_member(self, archive):
        if self.size < 0:
            raise ValueError('Invalid negative size in source archive member')
        count = getattr(archive, '_source_header_count', 0) + 1
        if count > MAX_ARCHIVE_MEMBERS:
            raise ValueError('Source archive exceeds member count limit')
        archive._source_header_count = count
        return super()._proc_member(archive)

    def _proc_sparse(self, *args):
        raise ValueError('Sparse source archive members are unsupported')

    def _apply_pax_info(self, headers, *args):
        # TarInfo silently turns malformed PAX numeric fields into zero.
        if 'size' in headers and not re.fullmatch(r'[0-9]+', headers['size']):
            raise ValueError('Invalid size in source archive PAX metadata')
        if any(key.startswith('GNU.sparse.') for key in headers):
            self._proc_sparse()
        return super()._apply_pax_info(headers, *args)

    # Reject each GNU/PAX sparse dialect before its parser reads sparse maps.
    _proc_gnusparse_00 = _proc_sparse
    _proc_gnusparse_01 = _proc_sparse
    _proc_gnusparse_10 = _proc_sparse


@contextmanager
def _open_archive(blob):
    """Read GitHub gzip tar with PAX/GNU names, no sparse files, and zero padding."""
    if len(blob) > MAX_ARCHIVE:
        raise ValueError('Source archive exceeds compressed size limit')
    try:
        with gzip.GzipFile(fileobj=io.BytesIO(blob), mode='rb') as decoded:
            bounded = _ExpandedArchiveReader(decoded)
            with tarfile.open(fileobj=bounded, mode='r|', tarinfo=_SourceTarInfo) as archive:
                yield archive
                if not getattr(archive, '_source_end_seen', False):
                    raise ValueError('Source archive is missing its end blocks')
                # Drain through tar's stream to include its buffered read-ahead.
                # GitHub archives have zero padding, not a second tar archive.
                while True:
                    padding = archive.fileobj.read(64 * 1024)
                    if not padding:
                        break
                    if padding.strip(b'\0'):
                        raise ValueError('Source archive has nonzero trailing data')
    except (OSError, EOFError, zlib.error) as error:
        raise ValueError('Invalid or truncated gzip source archive') from error
    except RecursionError as error:
        raise ValueError('Source archive metadata exceeds parser nesting limit') from error


def read_archive(blob, entry):
    selection_files(entry)
    files = {}
    total = 0
    seen = set()
    roots = set()
    with _open_archive(blob) as archive:
        for member in archive:
            parts = safe_path(member.name.rstrip('/')).parts
            roots.add(parts[0])
            if len(roots) != 1:
                raise ValueError('Archive has multiple roots')
            if len(parts) < 2:
                continue
            relative = '/'.join(parts[1:])
            required_paths = [entry['path']] + entry['license_files'] + list(entry.get('extra_files', {}))
            if (not member.isdir()
                    and any(path.startswith(relative + '/') or path == relative == entry['path']
                            for path in required_paths)):
                raise ValueError('Non-directory ancestor in selected archive path')
            targets = member_targets(relative, entry)
            if not targets:
                continue
            if member.isdir():
                if relative in entry.get('extra_files', {}) or relative in entry['license_files']:
                    raise ValueError('Selected extra/license must be a regular file')
                continue
            if not member.isfile():
                raise ValueError('Non-regular archive member: ' + member.name)
            if relative in seen:
                raise ValueError('Duplicate archive member: ' + relative)
            seen.add(relative)
            total += member.size * len(targets)
            if total > MAX_PAYLOAD:
                raise ValueError('Skill payload exceeds size limit')
            if len(files) + len(targets) > MAX_FILES:
                raise ValueError('Skill payload exceeds file count limit')
            stream = archive.extractfile(member)
            content = stream.read(member.size + 1)
            if len(content) != member.size:
                raise ValueError('Archive file size mismatch')
            for target in targets:
                add_selected(files, target, content)
    missing = (set(entry['license_files']) | set(entry.get('extra_files', {}))) - seen
    if missing:
        raise ValueError('Missing selected source file: ' + sorted(missing)[0])
    check_selected(files, entry)
    return files


def github_source(source):
    repo = source['repository']
    commit = source['commit']
    if not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', repo):
        raise ValueError('Invalid GitHub repository')
    if not re.fullmatch(r'[0-9a-f]{40}', commit):
        raise ValueError('Source must be pinned to a full commit SHA')
    return repo, commit


def fetch_bytes(url, limit):
    request = urllib.request.Request(url, headers={'User-Agent': 'personal-skill-catalog/1'})
    with urllib.request.urlopen(request, timeout=60) as response:
        blob = response.read(limit + 1)
    if len(blob) > limit:
        raise ValueError('Source response exceeds size limit')
    return blob


def download(source):
    repo, commit = github_source(source)
    url = 'https://codeload.github.com/' + repo + '/tar.gz/' + commit
    return fetch_bytes(url, MAX_ARCHIVE)


def read_remote_files(source, entry, cache=None):
    """Fetch only a pinned skill and notices, using a bounded complete Git tree."""
    selection_files(entry)
    repo, commit = github_source(source)
    cache = {} if cache is None else cache
    tree_key = ('tree', repo, commit)
    if tree_key not in cache:
        url = 'https://api.github.com/repos/' + repo + '/git/trees/' + commit + '?recursive=1'
        try:
            tree = json.loads(fetch_bytes(url, MAX_TREE))
        except (UnicodeError, json.JSONDecodeError) as error:
            raise ValueError('Invalid GitHub tree response') from error
        if (not isinstance(tree, dict) or tree.get('truncated') is not False
                or not isinstance(tree.get('tree'), list)
                or not isinstance(tree.get('sha'), str)
                or not re.fullmatch(r'[0-9a-f]{40}', tree['sha'])):
            raise ValueError('Missing or truncated GitHub tree')
        index = {}
        valid_modes = {'tree': {'040000'}, 'blob': {'100644', '100755', '120000'},
                       'commit': {'160000'}}
        for item in tree['tree']:
            if not isinstance(item, dict):
                raise ValueError('Invalid GitHub tree item')
            path, kind = item.get('path'), item.get('type')
            if (not isinstance(path, str) or not path or path.startswith('/')
                    or '\\' in path or any(p in ('', '.', '..') for p in path.split('/'))
                    or path in index or not isinstance(kind, str) or kind not in valid_modes
                    or not isinstance(item.get('mode'), str) or item['mode'] not in valid_modes[kind]
                    or not isinstance(item.get('sha'), str)
                    or not re.fullmatch(r'[0-9a-f]{40}', item['sha'])):
                raise ValueError('Invalid or duplicate GitHub tree path')
            if kind == 'blob' and (type(item.get('size')) is not int or item['size'] < 0):
                raise ValueError('Invalid GitHub blob size')
            index[path] = item
        cache[tree_key] = index
    index = cache[tree_key]
    for path in [entry['path']] + entry['license_files'] + list(entry.get('extra_files', {})):
        parts = safe_path(path).parts
        for length in range(1, len(parts)):
            ancestor = index.get('/'.join(parts[:length]))
            if ancestor is not None and ancestor['type'] != 'tree':
                raise ValueError('Non-directory ancestor in selected GitHub tree')
    root = index.get(entry['path'])
    if root is not None and root['type'] != 'tree':
        raise ValueError('Selected skill is not a GitHub tree directory')
    selected = {}
    required = set(entry['license_files']) | set(entry.get('extra_files', {}))
    if required - set(index):
        raise ValueError('Missing selected GitHub source file: ' + sorted(required - set(index))[0])
    for relative, item in index.items():
        targets = member_targets(relative, entry)
        if not targets:
            continue
        if item['type'] == 'tree':
            if relative in entry['license_files'] or relative in entry.get('extra_files', {}):
                raise ValueError('Selected extra/license must be a regular file')
            continue
        if item['type'] != 'blob' or item['mode'] not in ('100644', '100755'):
            raise ValueError('Non-regular selected GitHub file: ' + relative)
        safe_path(relative)
        for destination in targets:
            add_selected(selected, destination, (relative, item['size']))
    check_selected(selected, entry)
    if sum(size for _, size in selected.values()) > MAX_PAYLOAD:
        raise ValueError('Skill payload exceeds size limit')
    files = {}
    for target, (relative, size) in selected.items():
        key = ('file', repo, commit, relative)
        if key not in cache:
            url = ('https://raw.githubusercontent.com/' + repo + '/' + commit + '/'
                   + urllib.parse.quote(relative, safe='/'))
            blob = fetch_bytes(url, min(size, MAX_PAYLOAD))
            if len(blob) != size:
                raise ValueError('GitHub file size differs from pinned tree: ' + relative)
            cache[key] = blob
        files[target] = cache[key]
    return files


def skill_name(raw):
    text = raw.decode('utf-8')
    if not text.startswith('---\n') and not text.startswith('---\r\n'):
        raise ValueError('Missing skill frontmatter')
    frontmatter = text.split('---', 2)[1]
    match = re.search(r'^name:\s*[\"\']?([a-z0-9-]+)[\"\']?\s*$', frontmatter, re.M)
    if not match:
        raise ValueError('Invalid or missing skill name')
    return match.group(1)


def existing_payload(target):
    files = {}
    for root, directories, names in os.walk(target, followlinks=False):
        for name in directories + names:
            if is_link(Path(root) / name):
                raise ValueError('Installed skill contains a symlink; refusing to replace')
        for name in names:
            path = Path(root) / name
            if path.relative_to(target).as_posix() != RECEIPT:
                files[path.relative_to(target).as_posix()] = path.read_bytes()
    return files


def executable_contract(value):
    """Validate executable metadata without changing the byte-hash format."""
    if (not isinstance(value, list)
            or any(not isinstance(relative, str) for relative in value)
            or len(value) != len(set(value))):
        raise ValueError('Invalid executable file list')
    for relative in value:
        safe_path(relative)
        if relative == RECEIPT:
            raise ValueError('The installed receipt cannot be an executable payload')
    return value


def installed_executables(receipt):
    return validate_receipt(receipt)['executable_files']


def validate_receipt(receipt):
    """Require complete saved provenance before a copy can authorize reconciliation."""
    fields = {'id', 'repository', 'commit', 'path', 'sha256', 'installed_sha256',
              'executable_files'}
    if not isinstance(receipt, dict):
        raise ValueError('Installed receipt must be an object')
    missing = fields - set(receipt)
    if missing:
        raise ValueError('Installed receipt is missing the required ' + sorted(missing)[0] + ' contract')
    for field in ('id', 'path'):
        value = receipt[field]
        if not isinstance(value, str) or not value.strip():
            raise ValueError('Invalid installed receipt ' + field)
        safe_path(value)
    repository = receipt['repository']
    if (not isinstance(repository, str)
            or not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9_.-]*/[A-Za-z0-9][A-Za-z0-9_.-]*', repository)):
        raise ValueError('Invalid installed receipt repository')
    commit = receipt['commit']
    if commit is None:
        if repository != 'codemirket/harness':
            raise ValueError('Installed upstream receipt requires a pinned commit')
    elif not isinstance(commit, str) or not re.fullmatch(r'(?:[a-f0-9]{40}|[a-f0-9]{64})', commit):
        raise ValueError('Invalid installed receipt commit')
    for field in ('sha256', 'installed_sha256'):
        if not isinstance(receipt[field], str) or not re.fullmatch(r'[a-f0-9]{64}', receipt[field]):
            raise ValueError('Invalid installed receipt ' + field)
    executable_contract(receipt['executable_files'])
    return receipt


def executable_modes(target):
    """Snapshot POSIX execute bits only; Windows mode emulation is not evidence."""
    modes = {}
    if os.name == 'nt':
        return modes
    for root, directories, names in os.walk(target, followlinks=False):
        for name in directories + names:
            if is_link(Path(root) / name):
                raise ValueError('Installed skill contains a symlink; refusing to replace')
        for name in names:
            path = Path(root) / name
            info = path.stat()
            if not stat.S_ISREG(info.st_mode):
                raise ValueError('Non-regular installed payload: ' + str(path))
            modes[path.relative_to(target).as_posix()] = info.st_mode & 0o111
    return modes


def check_executable_modes(target, executable_files):
    declared = set(executable_contract(executable_files))
    modes = executable_modes(target)
    for relative in declared:
        path = target.joinpath(*safe_path(relative).parts)
        if not path.is_file() or (os.name != 'nt' and modes.get(relative) != 0o111):
            raise ValueError('Installed executable mode changed: ' + str(path))
    for relative, bits in modes.items():
        if bits and relative not in declared:
            raise ValueError('Undeclared executable mode in installed payload: ' + str(target / relative))


def supports_target(entry, target):
    # Existing catalog agents describe skill format families, not desktop apps.
    return target_registry.skill_family(target) in entry.get('agents', ['codex', 'claude'])


def check_destination(entry, project, agent):
    if entry['scope'] != 'project' or entry.get('delivery') not in ('upstream', 'local'):
        raise ValueError('This entry is not approved for automatic project installation')
    agent = target_registry.normalize(agent)
    if not supports_target(entry, agent):
        raise ValueError('Skill does not support this agent: ' + agent)
    project = project.expanduser().resolve(strict=True)
    if not project.is_dir():
        raise ValueError('Project must be an existing directory')
    selection_files(entry)
    name = entry['name']
    if not re.fullmatch(r'[a-z0-9]+(?:-[a-z0-9]+)*', name):
        raise ValueError('Unsafe installation name')
    safe_path(name)
    target_root = project / target_registry.project_skills_dir(agent)
    # Never follow a project metadata directory into a different checkout/home.
    for part in (target_root.parent, target_root):
        if is_link(part):
            raise ValueError('Refusing a symlinked project skill directory: ' + str(part))
    target = target_root / name
    if is_link(target):
        raise ValueError('Existing destination is a symlink; inspect it manually')
    if target.exists():
        receipt = target / RECEIPT
        prior = json.loads(receipt.read_text(encoding='utf-8')) if receipt.is_file() and not is_link(receipt) else {}
        if (isinstance(prior, dict) and prior.get('id') == entry['id']
                and payload_hash(existing_payload(target)) == prior.get('installed_sha256')
                == entry.get('installed_sha256', entry['sha256'])):
            prior_executables = installed_executables(prior)
            check_executable_modes(target, prior_executables)
            current_executables = executable_contract(entry.get('executable_files', []))
            if set(prior_executables) != set(current_executables):
                raise ValueError('Installed executable contract differs from catalog: ' + str(target))
            check_executable_modes(target, current_executables)
            return target, True
        raise ValueError('Destination exists; preserve/review it before changing: ' + str(target))
    if target_root.exists():
        for other in target_root.iterdir():
            manifest = other / 'SKILL.md'
            if manifest.is_file() and skill_name(manifest.read_bytes()) == name:
                raise ValueError('Skill name already registered: ' + str(other))
    return target, False


def adapt_payload(original, entry):
    files = dict(original)
    replacements = entry.get('replacements', [])
    if not isinstance(replacements, list):
        raise ValueError('replacements must be a list')
    for replacement in replacements:
        if not isinstance(replacement, dict) or set(replacement) != {'path', 'old', 'new', 'count'}:
            raise ValueError('Replacement must specify path, old, new, count')
        path, old, new, count = (replacement[key] for key in ('path', 'old', 'new', 'count'))
        if (not isinstance(path, str) or not isinstance(old, str) or not old
                or not isinstance(new, str) or type(count) is not int or count < 1):
            raise ValueError('Invalid literal replacement')
        safe_path(path)
        if path not in files:
            raise ValueError('Missing replacement file: ' + path)
        try:
            text = files[path].decode('utf-8')
        except UnicodeError as error:
            raise ValueError('Replacement file is not UTF-8: ' + path) from error
        if text.count(old) != count:
            raise ValueError('Replacement occurrence count differs: ' + path)
        files[path] = text.replace(old, new).encode('utf-8')
    if sum(map(len, files.values())) > MAX_PAYLOAD:
        raise ValueError('Adapted skill payload exceeds size limit')
    note = entry.get('adaptation')
    if note:
        text = files['SKILL.md'].decode('utf-8')
        boundary = text.find('\n---', 3)
        if boundary < 0:
            raise ValueError('Cannot locate skill frontmatter boundary')
        boundary += len('\n---')
        text = text[:boundary] + '\n\n## Personal catalog integration\n\n' + note + '\n' + text[boundary:]
        files['SKILL.md'] = text.encode('utf-8')
    return files


def prepare_payload(data, entry, source_tree=None, archive_cache=None):
    if entry['delivery'] == 'local':
        source = {'repository': 'codemirket/harness', 'commit': None}
        source_tree = ROOT
    else:
        source = data['sources'][entry['source']]
    if source_tree:
        files = read_local(source_tree, entry)
    elif source.get('fetch_mode') == 'files':
        files = read_remote_files(source, entry, archive_cache)
    else:
        if source.get('fetch_mode', 'archive') != 'archive':
            raise ValueError('Unsupported source fetch mode')
        key = (source['repository'], source['commit'])
        if archive_cache is not None:
            if key not in archive_cache:
                archive_cache[key] = download(source)
            blob = archive_cache[key]
        else:
            blob = download(source)
        files = read_archive(blob, entry)
    if sum(map(len, files.values())) > MAX_PAYLOAD:
        raise ValueError('Skill payload exceeds size limit')
    validate_paths(files)
    validate_paths(list(files) + [RECEIPT])
    if RECEIPT in files:
        raise ValueError('Upstream payload conflicts with receipt name')
    if 'SKILL.md' not in files or skill_name(files['SKILL.md']) != entry['name']:
        raise ValueError('Upstream skill name does not match catalog')
    if payload_hash(files) != entry['sha256']:
        raise ValueError('Payload differs from the reviewed content; installation stopped')
    files = adapt_payload(files, entry)
    if sum(map(len, files.values())) > MAX_PAYLOAD:
        raise ValueError('Adapted skill payload exceeds size limit')
    if skill_name(files['SKILL.md']) != entry['name']:
        raise ValueError('Adaptation changes the installed skill name')
    if payload_hash(files) != entry.get('installed_sha256', entry['sha256']):
        raise ValueError('Adapted payload differs from reviewed installed hash')
    for relative in executable_contract(entry.get('executable_files', [])):
        if relative not in files:
            raise ValueError('Missing executable file: ' + relative)
    return files, source


def install(data, entry, project, agent, source_tree=None, prepared=None):
    target, unchanged = check_destination(entry, project, agent)
    if unchanged:
        return 'Already installed and unchanged: ' + str(target)
    files, source = prepared if prepared is not None else prepare_payload(data, entry, source_tree)
    receipt = {'id': entry['id'], 'repository': source['repository'],
               'commit': source['commit'], 'path': entry['path'], 'sha256': entry['sha256'],
               'installed_sha256': entry.get('installed_sha256', entry['sha256']),
               'executable_files': list(entry.get('executable_files', []))}
    validate_receipt(receipt)
    target_root = target.parent
    target_root.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='.skill-catalog-', dir=target_root) as staging:
        for relative, content in files.items():
            path = Path(staging).joinpath(*safe_path(relative).parts)
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(content)
        if payload_hash(existing_payload(Path(staging))) != entry.get('installed_sha256', entry['sha256']):
            raise ValueError('Staged payload differs from reviewed content')
        (Path(staging) / RECEIPT).write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
        target.mkdir()  # Exclusive creation: a concurrently created destination fails.
        try:
            shutil.copytree(staging, target, dirs_exist_ok=True)
            for relative in entry.get('executable_files', []):
                executable = target.joinpath(*safe_path(relative).parts)
                if not executable.is_file():
                    raise ValueError('Missing executable file: ' + relative)
                executable.chmod(executable.stat().st_mode | 0o111)
        except BaseException:
            shutil.rmtree(target)
            raise
    return 'Installed ' + entry['id'] + ': ' + str(target)


def resolve_selection(data, profiles=(), identifiers=(), skip=()):
    """Resolve profiles and required companions; reject cycles and name clashes."""
    by_id = {entry['id']: entry for entry in data['skills']}
    selected = []
    visiting = set()
    seen = set()
    skipped = set(skip)
    if skipped - set(by_id):
        raise ValueError('Unknown skipped skill: ' + sorted(skipped - set(by_id))[0])

    def visit(identifier, required=False):
        if identifier in skipped:
            if required:
                raise ValueError('Cannot skip required companion: ' + identifier)
            return
        if identifier not in by_id:
            raise ValueError('Unknown catalog id: ' + identifier)
        if identifier in visiting:
            raise ValueError('Cyclic skill requirements: ' + identifier)
        if identifier in seen:
            return
        entry = by_id[identifier]
        if entry['scope'] != 'project' or entry.get('delivery') not in ('local', 'upstream'):
            raise ValueError('Selection is not installable: ' + identifier)
        visiting.add(identifier)
        for dependency in entry.get('requires', []):
            visit(dependency, required=True)
        visiting.remove(identifier)
        seen.add(identifier)
        selected.append(entry)

    for profile in profiles:
        if profile not in data.get('profiles', {}):
            raise ValueError('Unknown profile: ' + profile)
        for identifier in data['profiles'][profile]['skills']:
            visit(identifier)
    for identifier in identifiers:
        visit(identifier)
    names = {}
    for entry in selected:
        if entry['name'] in names:
            raise ValueError('Competing skill name: ' + names[entry['name']] + ' / ' + entry['id'])
        names[entry['name']] = entry['id']
        conflicts = seen.intersection(entry.get('conflicts', []))
        if conflicts:
            raise ValueError('Conflicting selections: ' + entry['id'] + ' / ' + sorted(conflicts)[0])
    return selected


def install_many(data, entries, project, agent, source_trees=None):
    """Preflight the whole selection and verify every payload before project writes."""
    agents = target_registry.names(agent)
    jobs = [(entry, target_agent) for entry in entries for target_agent in agents]
    states = [(entry, target_agent, *check_destination(entry, project, target_agent))
              for entry, target_agent in jobs]
    cache = {}
    prepared = {}
    for entry, _, _, unchanged in states:
        if not unchanged and entry['id'] not in prepared:
            source_tree = (source_trees or {}).get(entry.get('source'))
            prepared[entry['id']] = prepare_payload(data, entry, source_tree, cache)
    results = []
    # Later OS failures may leave earlier successful installs; never delete user
    # content to pretend this is a cross-directory filesystem transaction.
    for entry, target_agent, _, unchanged in states:
        results.append(install(data, entry, project, target_agent,
                               prepared=prepared.get(entry['id'])))
    return results


def matches(entry, query):
    fields = [entry.get(key, '') for key in ('id', 'name', 'description', 'source_description', 'category', 'source')]
    text = re.sub(r'[-_]', ' ', ' '.join(fields + entry.get('tags', [])).casefold())
    terms = re.sub(r'[-_]', ' ', query.casefold()).split()
    return all(term in text for term in terms)


def search_entries(data, indexed, query, source=None):
    """Search reviewed catalog metadata alongside the pinned discovery inventory."""
    by_id = {entry['id']: entry for entry in data['skills']}
    represented = set()
    rows = []
    for original in indexed:
        row = dict(original)
        identifiers = row.get('catalog_ids', [])
        represented.update(identifiers)
        curated = [by_id[value] for value in identifiers if value in by_id]
        if curated:
            row['source_description'] = row.get('description', '')
            row['description'] = ' '.join(dict.fromkeys(e.get('description', '') for e in curated))
            row['tags'] = list(dict.fromkeys(row.get('tags', []) + [
                value for entry in curated for value in
                [entry['id'], entry['name'], entry.get('category', '')] + entry.get('tags', [])]))
        rows.append(row)
    for entry in data['skills']:
        if entry['id'] in represented or entry.get('delivery') not in ('local', 'global-link'):
            continue
        rows.append(dict(entry, source=entry.get('source') or 'personal',
                         catalog_ids=[entry['id']],
                         installable_ids=[entry['id']] if entry['scope'] == 'project' else [],
                         review_status='authored'))
    results = [row for row in rows if matches(row, query)
               and (not source or row['source'] == source)]
    return sorted(results, key=lambda row: search_rank(row, query))


def search_rank(entry, query):
    """Prefer exact capability names/tags and reviewed choices over unreviewed leads."""
    normalized = ' '.join(re.sub(r'[-_]', ' ', query.casefold()).split())
    name = re.sub(r'[-_]', ' ', entry.get('name', '').casefold())
    named_match = all(term in name for term in normalized.split())
    tagged_match = any(normalized == re.sub(r'[-_]', ' ', tag.casefold())
                       for tag in entry.get('tags', []))
    status = {'authored': 0, 'installable-static-review': 0, 'manual': 1, 'manual-integration': 1,
              'indexed-only': 2, 'advertisement-only': 3}
    return (not (name == normalized or tagged_match),
            status.get(entry.get('review_status'), 2), name != normalized,
            not named_match, name, entry.get('source', ''))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    listing = commands.add_parser('list', help='List matching entries without installing anything')
    listing.add_argument('--scope', choices=['global', 'project', 'manual', 'excluded'])
    listing.add_argument('--category')
    listing.add_argument('--source')
    listing.add_argument('--query', default='')
    listing.add_argument('--json', action='store_true')
    inventory = commands.add_parser('search', help='Search the complete indexed source inventory; indexed does not mean installable')
    inventory.add_argument('query', nargs='?', default='')
    inventory.add_argument('--source')
    inventory.add_argument('--limit', type=int, default=30)
    inventory.add_argument('--json', action='store_true')
    commands.add_parser('profiles', help='List composable project capability profiles')
    for command in ('plan', 'install-profile'):
        selection = commands.add_parser(command, help='Preview or register a composed selection')
        selection.add_argument('--profile', action='append', default=[])
        selection.add_argument('--skill', action='append', default=[])
        selection.add_argument('--skip', action='append', default=[])
        selection.add_argument('--agent', choices=target_registry.CHOICES, default='codex')
        selection.add_argument('--project', type=Path, required=True)
    showing = commands.add_parser('show', help='Show provenance, fit, dependencies, and caveats')
    showing.add_argument('id')
    hashing = commands.add_parser('hash', help='Compute review hashes from a local checkout; does not change catalog approval')
    hashing.add_argument('id')
    hashing.add_argument('--source-tree', type=Path)
    installing = commands.add_parser('install', help='Register one pinned skill in an explicitly selected project')
    installing.add_argument('id')
    installing.add_argument('--agent', choices=target_registry.CHOICES, default='codex')
    installing.add_argument('--project', type=Path, required=True)
    installing.add_argument('--source-tree', type=Path, help='Use a local source checkout; reviewed content hash still required')
    args = parser.parse_args()
    data = load_catalog()
    if args.command == 'profiles':
        for name, profile in data.get('profiles', {}).items():
            print(f"{name} ({len(profile['skills'])} skills): {profile['description']}")
        return
    if args.command == 'search':
        indexed = json.loads(INDEX.read_text(encoding='utf-8'))['skills']
        results = search_entries(data, indexed, args.query, args.source)
        if args.limit < 1:
            raise ValueError('Search limit must be positive')
        results = results[:args.limit]
        if args.json:
            print(json.dumps(results, indent=2, ensure_ascii=False))
        else:
            for entry in results:
                status = entry.get('review_status', 'indexed-only') + ': ' + (', '.join(entry.get('catalog_ids', [])) or 'review before installation')
                print(f"{entry['source']}/{entry['name']} [{status}] {entry['description']}")
        return
    if args.command in ('plan', 'install-profile'):
        entries = resolve_selection(data, args.profile, args.skill, args.skip)
        if not entries:
            raise ValueError('Select at least one profile or skill')
        if args.command == 'plan':
            rows = []
            for entry in entries:
                agents = target_registry.names(args.agent)
                for agent in agents:
                    target, unchanged = check_destination(entry, args.project, agent)
                    rows.append({'id': entry['id'], 'agent': agent, 'destination': str(target),
                                 'state': 'unchanged' if unchanged else 'new',
                                 'dependencies': entry.get('dependencies', []),
                                 'caveats': entry.get('caveats', [])})
            print(json.dumps(rows, indent=2, ensure_ascii=False))
        else:
            for result in install_many(data, entries, args.project, args.agent):
                print(result)
        return
    if args.command == 'list':
        results = []
        for entry in data['skills']:
            if args.scope and entry['scope'] != args.scope:
                continue
            if args.category and entry['category'] != args.category:
                continue
            if args.source and entry.get('source') != args.source:
                continue
            if not matches(entry, args.query):
                continue
            results.append(entry)
        if args.json:
            print(json.dumps(results, indent=2, ensure_ascii=False))
        else:
            for entry in results:
                print(f"{entry['id']} [{entry['scope']}/{entry['category']}] {entry['description']}")
        return
    entry = next((item for item in data['skills'] if item['id'] == args.id), None)
    if entry is None:
        raise ValueError('Unknown catalog id: ' + args.id)
    if args.command == 'hash':
        if entry.get('delivery') not in ('local', 'upstream'):
            raise ValueError('Only installable selections have payload hashes')
        source_tree = (ROOT if entry['delivery'] == 'local'
                       else args.source_tree)
        if source_tree is None:
            raise ValueError('An upstream review requires --source-tree')
        files = read_local(source_tree, entry)
        print(json.dumps({'sha256': payload_hash(files),
                          'installed_sha256': payload_hash(adapt_payload(files, entry))}, indent=2))
    elif args.command == 'show':
        result = dict(entry)
        if entry.get('source'):
            result['source_record'] = data['sources'][entry['source']]
        print(json.dumps(result, indent=2, ensure_ascii=False))
    else:
        entries = resolve_selection(data, identifiers=[args.id])
        source_trees = {entry.get('source'): args.source_tree} if args.source_tree else None
        for result in install_many(data, entries, args.project, args.agent, source_trees):
            print(result)


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, tarfile.TarError) as error:
        print('Error: ' + str(error), file=sys.stderr)
        sys.exit(1)
