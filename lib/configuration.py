"""Lossless, dependency-free JSON/JSONC configuration merging (Python 3.9+).

Only selected value spans are replaced. Object members absent from the requested
configuration, comments, and all other existing text remain unchanged. No files
are read or written here; callers own backups and atomic installation.
"""
import json
import math
import re

_NUMBER = re.compile(r'-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?(?:[eE][+-]?[0-9]+)?')


class _Node:
    def __init__(self, value, start, end, members=None, trailing_comma=False):
        self.value = value
        self.start = start
        self.end = end
        self.members = members
        self.trailing_comma = trailing_comma


class _Parser:
    def __init__(self, text):
        if not isinstance(text, str):
            raise ValueError('Configuration must be text')
        self.text = text
        self.pos = 0

    def fail(self, message):
        raise ValueError('{} at character {}'.format(message, self.pos))

    def space(self):
        while self.pos < len(self.text):
            if self.text[self.pos] in ' \t\r\n':
                self.pos += 1
            elif self.text.startswith('//', self.pos):
                end = self.text.find('\n', self.pos + 2)
                self.pos = len(self.text) if end < 0 else end + 1
            elif self.text.startswith('/*', self.pos):
                end = self.text.find('*/', self.pos + 2)
                if end < 0:
                    self.fail('Unterminated block comment')
                self.pos = end + 2
            else:
                break

    def string(self):
        try:
            value, end = json.JSONDecoder().raw_decode(self.text, self.pos)
        except (ValueError, RecursionError):
            self.fail('Invalid JSON string')
        if not isinstance(value, str):
            self.fail('Expected a quoted object key')
        self.pos = end
        return value

    def value(self, depth=0):
        if depth > 200:
            self.fail('Configuration nesting exceeds 200 levels')
        self.space()
        start = self.pos
        if self.pos >= len(self.text):
            self.fail('Expected a JSON value')
        char = self.text[self.pos]
        if char == '{':
            self.pos += 1
            members = {}
            result = {}
            trailing = False
            self.space()
            while self.pos < len(self.text) and self.text[self.pos] != '}':
                if self.text[self.pos] != '"':
                    self.fail('Expected a quoted object key')
                key = self.string()
                if key in members:
                    self.fail('Duplicate object key {!r}'.format(key))
                self.space()
                if self.pos >= len(self.text) or self.text[self.pos] != ':':
                    self.fail('Expected colon after object key')
                self.pos += 1
                node = self.value(depth + 1)
                members[key] = node
                result[key] = node.value
                self.space()
                if self.pos < len(self.text) and self.text[self.pos] == ',':
                    self.pos += 1
                    self.space()
                    trailing = True
                else:
                    trailing = False
                    break
            if self.pos >= len(self.text) or self.text[self.pos] != '}':
                self.fail('Expected closing object brace or comma')
            self.pos += 1
            return _Node(result, start, self.pos, members, trailing)
        if char == '[':
            self.pos += 1
            result = []
            self.space()
            while self.pos < len(self.text) and self.text[self.pos] != ']':
                result.append(self.value(depth + 1).value)
                self.space()
                if self.pos < len(self.text) and self.text[self.pos] == ',':
                    self.pos += 1
                    self.space()
                else:
                    break
            if self.pos >= len(self.text) or self.text[self.pos] != ']':
                self.fail('Expected closing array bracket or comma')
            self.pos += 1
            return _Node(result, start, self.pos)
        if char == '"':
            result = self.string()
        else:
            result = None
            for literal, value in [('true', True), ('false', False), ('null', None)]:
                if self.text.startswith(literal, self.pos):
                    result = value
                    self.pos += len(literal)
                    break
            else:
                match = _NUMBER.match(self.text, self.pos)
                if not match:
                    self.fail('Invalid JSON value')
                result = json.loads(match.group())
                if isinstance(result, float) and not math.isfinite(result):
                    self.fail('Non-finite JSON number')
                self.pos = match.end()
        return _Node(result, start, self.pos)

    def document(self):
        node = self.value()
        self.space()
        if self.pos != len(self.text):
            self.fail('Unexpected content after configuration')
        if node.members is None:
            self.fail('Configuration root must be an object')
        return node


def parse_json(text):
    """Parse an object with JSONC comments/trailing commas; reject ambiguity."""
    return _Parser(text).document().value


def _check_desired(value, depth=0):
    if depth > 200:
        raise ValueError('Desired configuration nesting exceeds 200 levels')
    if isinstance(value, dict):
        if any(not isinstance(key, str) for key in value):
            raise ValueError('Desired configuration keys must be strings')
        for item in value.values():
            _check_desired(item, depth + 1)
    elif isinstance(value, list):
        for item in value:
            _check_desired(item, depth + 1)
    elif value is not None and type(value) not in (str, bool, int, float):
        raise ValueError('Desired configuration must contain JSON values')
    elif isinstance(value, float) and not math.isfinite(value):
        raise ValueError('Desired configuration contains a non-finite number')


def _equal(left, right):
    # Python considers True == 1, which would skip a material JSON type change.
    if isinstance(left, bool) != isinstance(right, bool):
        return False
    if isinstance(left, dict) and isinstance(right, dict):
        return left.keys() == right.keys() and all(_equal(left[k], right[k]) for k in left)
    if isinstance(left, list) and isinstance(right, list):
        return len(left) == len(right) and all(_equal(a, b) for a, b in zip(left, right))
    return left == right


def _leaves(value, path):
    if isinstance(value, dict) and value:
        return [item for key, child in value.items()
                for item in _leaves(child, path + (key,))]
    return ['.'.join(path)]


def merge_json(text, desired_dict):
    """Return (merged text, changed dotted paths), preserving unrelated bytes.

    Missing/empty files can be represented by empty or whitespace-only text.
    Existing object/non-object collisions raise ValueError: silently replacing
    an object would erase settings owned by the user or a different installer.
    """
    if not isinstance(desired_dict, dict):
        raise ValueError('Desired configuration root must be an object')
    _check_desired(desired_dict)
    if not isinstance(text, str):
        raise ValueError('Configuration must be text')
    if not text.strip():
        if not desired_dict:
            return text, []
        new_text = text + json.dumps(desired_dict, ensure_ascii=False, indent=2) + '\n'
        parse_json(new_text)
        return new_text, _leaves(desired_dict, ())
    root = _Parser(text).document()
    edits = []
    changed = []
    newline = '\r\n' if '\r\n' in text else '\n'

    def merge(node, desired, path):
        missing = {}
        for key, value in desired.items():
            child_path = path + (key,)
            if key not in node.members:
                missing[key] = value
                changed.extend(_leaves(value, child_path))
                continue
            current = node.members[key]
            if isinstance(value, dict) != (current.members is not None):
                raise ValueError('Object/value conflict at {}'.format('.'.join(child_path)))
            if isinstance(value, dict):
                merge(current, value, child_path)
            elif not _equal(current.value, value):
                edits.append((current.start, current.end,
                              json.dumps(value, ensure_ascii=False, allow_nan=False)))
                changed.append('.'.join(child_path))
        if not missing:
            return
        close = node.end - 1
        line_start = text.rfind('\n', 0, close) + 1
        close_indent = text[line_start:close]
        own_line = not close_indent.strip()
        # Use the closing brace's indentation, or the containing line's indent
        # for inline objects. Existing whitespace/comments are never rewritten.
        if not own_line:
            start_line = text.rfind('\n', 0, node.start) + 1
            close_indent = re.match(r'[ \t]*', text[start_line:node.start]).group()
        indent = close_indent + '  '
        if node.members:
            first = next(iter(node.members.values()))
            first_line = text.rfind('\n', 0, first.start) + 1
            match = re.match(r'[ \t]*', text[first_line:first.start])
            if first_line > node.start and len(match.group()) > len(close_indent):
                indent = match.group()
            if not node.trailing_comma:
                last = next(reversed(node.members.values()))
                edits.append((last.end, last.end, ','))
        pieces = []
        for key, value in missing.items():
            rendered = json.dumps(value, ensure_ascii=False, indent=2, allow_nan=False)
            rendered = rendered.replace('\n', newline + indent)
            pieces.append(indent + json.dumps(key, ensure_ascii=False) + ': ' + rendered)
        addition = (',' + newline).join(pieces)
        if node.trailing_comma:
            addition += ','
        if own_line:
            edits.append((line_start, line_start, addition + newline))
        else:
            edits.append((close, close, newline + addition + newline + close_indent))

    merge(root, desired_dict, ())
    if not edits:
        return text, []
    output = text
    for _, (start, end, replacement) in sorted(
            enumerate(edits), key=lambda item: (item[1][0], item[1][1], item[0]), reverse=True):
        output = output[:start] + replacement + output[end:]
    parse_json(output)
    return output, changed
