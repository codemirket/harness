"""Public SQLite development checks; no production or crash-durability claim."""
import importlib.util
import json
import sqlite3
import subprocess
import sys
import tempfile
from pathlib import Path


def source_rows(conn):
    return conn.execute('SELECT id,email,display_name,notes FROM customers ORDER BY id').fetchall()


def keys(conn):
    return conn.execute('SELECT id,email_key FROM customers ORDER BY id').fetchall()


def correct_keys(conn):
    return all(key == email.strip(' ').lower() for email, key in conn.execute('SELECT email,email_key FROM customers'))


def state(conn):
    return {'rows': source_rows(conn), 'keys': keys(conn),
            'audit': conn.execute('SELECT * FROM email_audit ORDER BY id').fetchall(),
            'unrelated': conn.execute('SELECT * FROM unrelated_settings ORDER BY name').fetchall()}


def main(root):
    checks = []

    def check(name, passed, detail):
        checks.append({'id': name, 'status': 'passed' if passed else 'failed', 'detail': detail})

    source = root / 'migration.py'
    try:
        spec = importlib.util.spec_from_file_location('candidate_migration', source)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
    except Exception as exc:
        return [{'id': 'module', 'status': 'failed', 'detail': f'Import failed: {type(exc).__name__}: {str(exc)[:100]}'}]

    with tempfile.TemporaryDirectory(prefix='backfill-eval-') as temporary:
        db = Path(temporary) / 'customers.db'
        conn = sqlite3.connect(db)
        try:
            conn.executescript((root / 'schema.sql').read_text(encoding='utf-8'))
            original = source_rows(conn)
            objects = dict(conn.execute("SELECT name,sql FROM sqlite_master WHERE name IN ('customers_display_name','preserve_email_audit','email_audit','unrelated_settings')"))
            module.expand(conn)
            columns = conn.execute('PRAGMA table_info(customers)').fetchall()
            added = [column for column in columns if column[1] == 'email_key']
            check('additive-expansion', len(added) == 1 and added[0][2].upper() == 'TEXT' and added[0][3] == 0
                  and source_rows(conn) == original and keys(conn) == [(row[0], None) for row in original]
                  and not conn.in_transaction,
                  'Nullable key added; source rows preserved and existing rows remain pending')
            before = state(conn)
            module.expand(conn)
            check('repeat-expansion', state(conn) == before and not conn.in_transaction,
                  'Repeated expansion leaves rows and audit unchanged')
            invalid_ok = True
            for value in (0, -1, True, 1.5, '2', None):
                before = state(conn)
                try:
                    module.backfill_batch(conn, value)
                    invalid_ok = False
                except ValueError:
                    pass
                except Exception:
                    invalid_ok = False
                invalid_ok = invalid_ok and state(conn) == before and not conn.in_transaction
            check('batch-validation', invalid_ok, 'Invalid batch sizes raise ValueError before database changes')
            count = module.backfill_batch(conn, 2)
            observer = sqlite3.connect(db)
            observed = keys(observer)
            observer.close()
            expected_first = [(r[0], r[1].strip(' ').lower() if i < 2 else None) for i, r in enumerate(original)]
            check('bounded-durable-batch', type(count) is int and count == 2 and observed == expected_first
                  and source_rows(conn) == original and not conn.in_transaction,
                  'Exactly the first two IDs are complete and visible to a new connection after return')

            # An ordinary old application connection must work while old rows remain pending.
            writer = sqlite3.connect(db)
            try:
                writer.execute("INSERT INTO customers VALUES (5,' NEW@EXAMPLE.TEST ','New','legacy insert',NULL)")
                # INSERT with the old column list is the actual compatibility contract.
                writer.execute("INSERT INTO customers(id,email,display_name,notes) VALUES (700,' LATE@EXAMPLE.TEST ','Late','legacy list')")
                writer.execute("UPDATE customers SET email=' EDITED@EXAMPLE.TEST ' WHERE id=10")
                writer.execute("UPDATE customers SET email=' PENDING@EXAMPLE.TEST ' WHERE id=500")
                writer.execute("INSERT INTO customers(id,email,display_name,notes,email_key) VALUES (600,' Both@EXAMPLE.TEST ','Both','new writer','both@example.test')")
                writer.execute("UPDATE customers SET email=' Paired@EXAMPLE.TEST ', email_key='paired@example.test' WHERE id=20")
                writer.execute("UPDATE customers SET email_key='eve@example.test' WHERE id=300")
                writer.commit()
                subset = writer.execute('SELECT id,email,email_key FROM customers WHERE id IN (5,10,20,300,500,600,700) ORDER BY id').fetchall()
                check('overlapping-writers', len(subset) == 7 and all(key == email.strip(' ').lower() for _, email, key in subset),
                      'Legacy, paired and pending-key writes work on an independent connection during backfill')
                unchanged = state(writer)
                refusal_ok = True
                for sql in ("INSERT INTO customers(id,email,display_name,email_key) VALUES (900,'valid@example.test','Bad','wrong@example.test')",
                            "UPDATE customers SET email_key='wrong@example.test' WHERE id=20",
                            "UPDATE customers SET email_key=NULL WHERE id=20",
                            "UPDATE customers SET email='new@example.test', email_key='wrong@example.test' WHERE id=20"):
                    try:
                        writer.execute(sql)
                        writer.commit()
                        refusal_ok = False
                    except sqlite3.DatabaseError:
                        writer.rollback()
                    refusal_ok = refusal_ok and state(writer) == unchanged
                check('reject-inconsistent-key', refusal_ok, 'Incorrect explicit new keys fail without data changes')
            finally:
                writer.close()

            # Fail the second row of the next batch. ABORT only reverts its SQL statement,
            # so row-by-row implementations must also roll back the preceding update.
            conn.execute("CREATE TRIGGER injected_failure BEFORE UPDATE OF email_key ON customers WHEN NEW.id=40 BEGIN SELECT RAISE(ABORT,'evaluation injected write failure'); END")
            conn.commit()
            before = state(conn)
            failed = False
            try:
                module.backfill_batch(conn, 2)
            except sqlite3.DatabaseError:
                failed = True
            check('atomic-failure', failed and state(conn) == before and not conn.in_transaction,
                  'Injected statement failure propagates and rolls back the entire pending batch')
            if conn.in_transaction:
                conn.rollback()
            conn.execute('DROP TRIGGER injected_failure')
            conn.commit()
            conn.close()

            # Successful batch must commit before return; exit bypasses Python cleanup.
            worker = '''import importlib.util, os, sqlite3, sys
spec = importlib.util.spec_from_file_location("restart_migration", sys.argv[1])
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
c = sqlite3.connect(sys.argv[2])
m.expand(c)
count = m.backfill_batch(c, 2)
os._exit(23 if type(count) is int and count == 2 and not c.in_transaction else 24)
'''
            result = subprocess.run([sys.executable, '-I', '-c', worker, str(source), str(db)],
                                    cwd=root, capture_output=True, timeout=5)
            conn = sqlite3.connect(db)
            middle = dict(keys(conn))
            check('process-restart', result.returncode == 23 and middle.get(35) == 'alice@example.test'
                  and middle.get(40) == 'carol@example.test' and middle.get(90) is None,
                  'New process resumes the next two rows; abrupt exit after return retains exactly the committed batch')
            counts = []
            for _ in range(20):
                before_pending = [r[0] for r in conn.execute('SELECT id FROM customers WHERE email_key IS NULL ORDER BY id')]
                count = module.backfill_batch(conn, 1)
                counts.append(count)
                if type(count) is not int or count not in (0, 1) or conn.in_transaction:
                    break
                if count == 0:
                    break
                expected_id = before_pending[0] if before_pending else None
                if expected_id is None or dict(keys(conn))[expected_id] is None:
                    break
            check('finish-resumable', counts and counts[-1] == 0 and all(type(n) is int and n in (0, 1) for n in counts)
                  and correct_keys(conn) and not conn.in_transaction,
                  'Further bounded calls finish all pending rows, preserving duplicate normalized keys')
            expected_rows = {r[0]: r for r in original}
            expected_rows[10] = (10, ' EDITED@EXAMPLE.TEST ', 'Alice', 'retain spelling')
            expected_rows[20] = (20, ' Paired@EXAMPLE.TEST ', 'Bob', 'paid account')
            expected_rows[500] = (500, ' PENDING@EXAMPLE.TEST ', 'Frank', 'late row')
            expected_rows.update({5: (5, ' NEW@EXAMPLE.TEST ', 'New', 'legacy insert'),
                                  600: (600, ' Both@EXAMPLE.TEST ', 'Both', 'new writer'),
                                  700: (700, ' LATE@EXAMPLE.TEST ', 'Late', 'legacy list')})
            current_objects = dict(conn.execute("SELECT name,sql FROM sqlite_master WHERE name IN ('customers_display_name','preserve_email_audit','email_audit','unrelated_settings')"))
            audit = conn.execute('SELECT customer_id,old_email,new_email FROM email_audit ORDER BY id').fetchall()
            check('data-and-schema-preservation', source_rows(conn) == [expected_rows[k] for k in sorted(expected_rows)]
                  and current_objects == objects and conn.execute('SELECT * FROM unrelated_settings').fetchall() == [('billing-mode', 'legacy')]
                  and audit == [(10, original[0][1], ' EDITED@EXAMPLE.TEST '), (500, original[-1][1], ' PENDING@EXAMPLE.TEST '), (20, original[1][1], ' Paired@EXAMPLE.TEST ')],
                  'Only intended source edits exist; original index, trigger, audit and unrelated data are preserved')
            before = state(conn)
            module.expand(conn)
            empty = module.backfill_batch(conn, 3)
            check('completed-idempotence', type(empty) is int and empty == 0 and state(conn) == before and not conn.in_transaction,
                  'Repeated expansion/backfill after completion changes no rows or audit records')
        except Exception as exc:
            check('migration-execution', False, f'{type(exc).__name__}: {str(exc)[:140]}')
        finally:
            conn.close()

        # Different IDs/values prevent fixture-only hardcoding from earning a pass.
        other = sqlite3.connect(Path(temporary) / 'other.db')
        try:
            other.executescript((root / 'schema.sql').read_text(encoding='utf-8'))
            other.execute('DELETE FROM customers')
            alternate = [(2, ' SAME@EXAMPLE.TEST ', 'One', 'keep'),
                         (17, 'same@example.test', 'Two', 'both survive'),
                         (4201, '\tTabbed@EXAMPLE.TEST\t', 'Three', 'tabs stay'),
                         (8000, '  SPACED@EXAMPLE.TEST   ', 'Four', '')]
            other.executemany('INSERT INTO customers(id,email,display_name,notes) VALUES (?,?,?,?)', alternate)
            other.commit()
            module.expand(other)
            counts = [module.backfill_batch(other, 3), module.backfill_batch(other, 3), module.backfill_batch(other, 3)]
            check('alternate-data', counts == [3, 1, 0] and all(type(n) is int for n in counts)
                  and source_rows(other) == alternate and correct_keys(other) and not other.in_transaction,
                  'Arbitrary sparse IDs, equal keys and ASCII trim semantics work on a second database')
        except Exception as exc:
            check('alternate-data', False, f'{type(exc).__name__}: {str(exc)[:120]}')
        finally:
            other.close()
    note = root / 'migration.md'
    check('migration-note', note.is_file() and bool(note.read_text(encoding='utf-8').strip()),
          'Migration note exists; operational reasoning and evidence require qualitative review')
    return checks


if __name__ == '__main__':
    try:
        result = main(Path(sys.argv[1]).resolve()) if len(sys.argv) == 2 else [{'id': 'usage', 'status': 'failed', 'detail': 'Expected workspace path'}]
    except Exception as exc:
        result = [{'id': 'verifier-error', 'status': 'failed', 'detail': f'{type(exc).__name__}: {str(exc)[:120]}'}]
    print(json.dumps({'checks': result}, separators=(',', ':')))
    sys.exit(0 if all(item['status'] == 'passed' for item in result) else 1)
