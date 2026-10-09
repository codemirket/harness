"""Lossless configuration updates, including JSONC used by Zed."""
import unittest

from lib.configuration import merge_json, parse_json


class ConfigurationTests(unittest.TestCase):
    def test_noop_is_byte_exact_with_urls_escapes_comments_and_trailing_commas(self):
        text = ('// prefix\r\n{\r\n  "url": "https://example.com/a//b",\r\n'
                '  "text": "escaped \\\" // /* not a comment */", /* keep */\r\n'
                '  "nested": {"a": [1, 2,],},\r\n}\r\n// suffix\r\n')
        self.assertEqual(merge_json(text, parse_json(text)), (text, []))

    def test_replaces_only_selected_leaf_span(self):
        text = '{\n  "nested": {"theme": "old", /* user */ "other": 1},\n  "private": "stay"\n}\n'
        output, paths = merge_json(text, {'nested': {'theme': 'new'}})
        self.assertEqual(output, text.replace('"old"', '"new"'))
        self.assertEqual(paths, ['nested.theme'])

    def test_missing_nested_objects_and_empty_objects(self):
        output, paths = merge_json('{"user": 1}', {'agent': {'models': {'x': 2}, 'empty': {}}})
        self.assertEqual(parse_json(output), {'user': 1, 'agent': {'models': {'x': 2}, 'empty': {}}})
        self.assertEqual(paths, ['agent.models.x', 'agent.empty'])
        self.assertEqual(merge_json(output, {'agent': {'models': {'x': 2}, 'empty': {}}}), (output, []))

    def test_empty_object_with_comments(self):
        for text in ['{}', '{ }', '{ /* preserve */ }', '{\n  // preserve\n}\n']:
            with self.subTest(text=text):
                output, paths = merge_json(text, {'a': 1, 'b': 2})
                self.assertEqual(parse_json(output), {'a': 1, 'b': 2})
                self.assertEqual(paths, ['a', 'b'])
                if 'preserve' in text:
                    self.assertIn('preserve', output)

    def test_add_after_comment_with_or_without_trailing_comma(self):
        for comma in ['', ',']:
            text = '{\n    "a": 1' + comma + ' // preserve\n}\n'
            output, _ = merge_json(text, {'b': 2})
            self.assertEqual(parse_json(output), {'a': 1, 'b': 2})
            self.assertIn('// preserve\n', output)
            self.assertIn('    "b": 2', output)

    def test_simultaneous_nested_edits_do_not_shift_spans(self):
        text = '{"a":{"x":1}, "b":{"y":2}, "c":3}'
        output, paths = merge_json(text, {'a': {'x': 10, 'new': True}, 'b': {'z': None}, 'd': []})
        self.assertEqual(parse_json(output), {'a': {'x': 10, 'new': True}, 'b': {'y': 2, 'z': None}, 'c': 3, 'd': []})
        self.assertEqual(paths, ['a.x', 'a.new', 'b.z', 'd'])

    def test_boolean_and_number_are_distinct(self):
        output, paths = merge_json('{"a":true,"b":[false]}', {'a': 1, 'b': [0]})
        self.assertEqual(output, '{"a":1,"b":[0]}')
        self.assertEqual(paths, ['a', 'b'])

    def test_numbers_null_and_arrays(self):
        text = '{"a":-1.25e2,"b":null,"c":[1,2]}'
        self.assertEqual(parse_json(text), {'a': -125.0, 'b': None, 'c': [1, 2]})
        output, paths = merge_json(text, {'a': 0.5, 'b': 'value', 'c': False})
        self.assertEqual(parse_json(output), {'a': 0.5, 'b': 'value', 'c': False})
        self.assertEqual(paths, ['a', 'b', 'c'])

    def test_object_collisions_fail_in_both_directions(self):
        for text, desired in [('{"a":{ "owned": 1 }}', {'a': 1}),
                              ('{"a":null}', {'a': {}}), ('{"a":[]}', {'a': {'b': 1}})]:
            with self.subTest(text=text), self.assertRaisesRegex(ValueError, 'conflict at a'):
                merge_json(text, desired)

    def test_invalid_config_fails_even_for_empty_desired(self):
        invalid = ['{"a":1,"a":2}', '{"a":{"x":1,"x":2}}',
                   '{"a":1,"\\u0061":2}', '{"x":[{"a":1,"a":2}]}',
                   '{"x":01}', '{"x":1.}', '{"x":NaN}', '{"x":1e999}',
                   '{"x":true false}', '{"x":1,,}', '{"x":[1,,]}',
                   '{"x":1', '{/* unterminated}', '{} garbage', '[]', 'null',
                   '{unquoted:1}', '{"x":"bad\\q"}', '{"x": "line\nbreak"}']
        for text in invalid:
            with self.subTest(text=text), self.assertRaises(ValueError):
                merge_json(text, {})

    def test_desired_must_be_json_object(self):
        for desired in [[], None, {1: 'x'}, {'a': float('nan')}, {'a': object()}, {'a': (1, 2)}]:
            with self.subTest(desired=desired), self.assertRaises(ValueError):
                merge_json('{}', desired)

    def test_empty_input_and_crlf(self):
        self.assertEqual(merge_json('', {}), ('', []))
        output, paths = merge_json('', {'x': {'a': 1}})
        self.assertEqual(parse_json(output), {'x': {'a': 1}})
        self.assertEqual(paths, ['x.a'])
        text = '{\r\n  "x": 1\r\n}\r\n'
        output, _ = merge_json(text, {'y': {'a': 2}})
        self.assertNotIn('\n', output.replace('\r\n', ''))


if __name__ == '__main__':
    unittest.main()
