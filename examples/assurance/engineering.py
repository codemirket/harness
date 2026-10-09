"""Synthetic local assurance fixture, not a production service or performance claim.

Run with Python standard library. Optional --output writes a NEW raw JSON report.
All databases are temporary. No network, dependency installation or host changes.
"""
import argparse
import concurrent.futures
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import random
import sqlite3
import statistics
import tempfile
import threading
import time

class Fault(Exception):
    pass

class Service:
    def __init__(self, path):
        self.path = path
        with self.connect() as db:
            db.executescript('''CREATE TABLE accounts(tenant TEXT, id TEXT, balance INTEGER CHECK(balance >= 0), PRIMARY KEY(tenant,id));
CREATE TABLE receipts(tenant TEXT, key TEXT, account TEXT, amount INTEGER, balance INTEGER, PRIMARY KEY(tenant,key));
INSERT INTO accounts VALUES ('a','credit',1000),('b','credit',1000);''')

    def connect(self):
        return sqlite3.connect(self.path, timeout=0.15, isolation_level=None)

    def debit(self, actor_tenant, account_tenant, account, amount, key, fail=False):
        # Caller identity is trusted service context; authentication is out of scope.
        if actor_tenant != account_tenant:
            raise Fault('forbidden')
        if type(amount) is not int or not 0 < amount <= 1000000 or not isinstance(key,str) or not 0 < len(key) <= 64:
            raise Fault('invalid')
        db = self.connect()
        try:
            db.execute('BEGIN IMMEDIATE')
            prior = db.execute('SELECT account,amount,balance FROM receipts WHERE tenant=? AND key=?',(actor_tenant,key)).fetchone()
            if prior:
                if prior[:2] != (account,amount):
                    raise Fault('conflict')
                db.execute('COMMIT')
                return {'key':key,'balance':prior[2]}
            changed = db.execute('UPDATE accounts SET balance=balance-? WHERE tenant=? AND id=? AND balance>=?',(amount,actor_tenant,account,amount)).rowcount
            if changed != 1:
                raise Fault('unavailable-credit')
            if fail:
                raise Fault('injected-before-receipt')
            balance = db.execute('SELECT balance FROM accounts WHERE tenant=? AND id=?',(actor_tenant,account)).fetchone()[0]
            db.execute('INSERT INTO receipts VALUES(?,?,?,?,?)',(actor_tenant,key,account,amount,balance))
            db.execute('COMMIT')
            return {'key':key,'balance':balance}
        except sqlite3.OperationalError as exc:
            if db.in_transaction:
                db.execute('ROLLBACK')
            if 'locked' in str(exc):
                raise Fault('busy') from exc
            raise
        except Exception:
            if db.in_transaction:
                db.execute('ROLLBACK')
            raise
        finally:
            db.close()

    def invariant(self):
        db = self.connect()
        try:
            rows = db.execute('SELECT a.tenant,a.balance,COALESCE((SELECT SUM(amount) FROM receipts r WHERE r.tenant=a.tenant AND r.account=a.id),0) FROM accounts a').fetchall()
            assert all(balance >= 0 and balance+debits == 1000 for _,balance,debits in rows), rows
            return rows
        finally:
            db.close()

def expect_fault(kind, fn):
    try:
        fn()
    except Fault as exc:
        assert str(exc) == kind, (kind,str(exc))
    else:
        raise AssertionError('expected '+kind)

def verify_service(root):
    s = Service(root/'service.sqlite')
    checks = []
    command = lambda amount,key,**kw:s.debit('a','a','credit',amount,key,**kw)
    expected = {'key':'A','balance':700}
    assert command(300,'A') == expected
    checks.append('exact debit result')
    # Original response can disappear; reconnecting replay uses durable receipt.
    assert command(300,'A') == expected
    checks.append('lost-response durable replay')
    expect_fault('conflict',lambda:command(301,'A'))
    checks.append('same-key changed payload rejected')
    expect_fault('forbidden',lambda:s.debit('b','a','credit',1,'A'))
    checks.append('tenant denial before receipt lookup')
    expect_fault('invalid',lambda:command(True,'boolean'))
    checks.append('boolean amount rejected')
    expect_fault('unavailable-credit',lambda:command(800,'B'))
    checks.append('insufficient balance no mutation')
    expect_fault('injected-before-receipt',lambda:command(20,'C',fail=True))
    assert s.invariant()[0] == ('a',700,300)
    checks.append('mid-transaction rollback and invariant')
    barrier = threading.Barrier(2)
    def race(key):
        barrier.wait()
        try:
            return command(500,key)
        except Fault as exc:
            return str(exc)
    with concurrent.futures.ThreadPoolExecutor(2) as pool:
        results = list(pool.map(race,['D','E']))
    assert sum(isinstance(r,dict) for r in results) == 1,results
    assert 'unavailable-credit' in results,results
    checks.append('concurrent distinct commands cannot overdraw')
    barrier = threading.Barrier(2)
    def replay(_):
        barrier.wait()
        return command(50,'F')
    with concurrent.futures.ThreadPoolExecutor(2) as pool:
        results = list(pool.map(replay,range(2)))
    assert results == [{'key':'F','balance':150}]*2,results
    checks.append('concurrent identical command has one effect')
    lock = s.connect()
    lock.execute('BEGIN IMMEDIATE')
    start=time.perf_counter()
    try:
        expect_fault('busy',lambda:command(1,'G'))
    finally:
        lock.execute('ROLLBACK')
        lock.close()
    elapsed=time.perf_counter()-start
    assert elapsed < 2.0,elapsed
    checks.append('writer lock produces bounded busy failure')
    assert command(300,'A') == expected  # original balance, not current balance
    checks.append('replay keeps original receipt after later commands')
    rows=s.invariant()
    assert rows == [('a',150,850),('b',1000,0)],rows
    checks.append('reopened storage conserves credit across tenants')
    return {'checks':checks,'count':len(checks),'final_rows':rows,'busy_wait_ms':round(elapsed*1000,3)}

def benchmark(root):
    rng=random.Random(42)
    rows=[(i, 'hot' if i%2==0 else 't'+str(i%101),i//3,i%999) for i in range(60000)]
    dbs={}
    for variant in ['baseline','indexed']:
        path=root/(variant+'.sqlite')
        db=sqlite3.connect(path)
        db.execute('CREATE TABLE history(id INTEGER PRIMARY KEY, tenant TEXT, created INTEGER, amount INTEGER)')
        db.executemany('INSERT INTO history VALUES(?,?,?,?)',rows)
        if variant=='indexed':
            db.execute('CREATE INDEX history_tenant_time ON history(tenant,created DESC,id DESC)')
        db.commit()
        dbs[variant]=db
    sql='SELECT id,created,amount FROM history WHERE tenant=? ORDER BY created DESC,id DESC LIMIT 20'
    params=['hot','t1','t73','missing']
    expected={p:sorted([(i,c,a) for i,t,c,a in rows if t==p],key=lambda x:(x[1],x[0]),reverse=True)[:20] for p in params}
    samples={v:[] for v in dbs}
    plans={v:db.execute('EXPLAIN QUERY PLAN '+sql,('hot',)).fetchall() for v,db in dbs.items()}
    for db in dbs.values():
        for _ in range(3):
            for p in params:
                assert db.execute(sql,(p,)).fetchall()==expected[p]
    for _ in range(40):
        order=list(dbs)
        rng.shuffle(order)
        for variant in order:
            for p in params:
                start=time.perf_counter_ns()
                result=dbs[variant].execute(sql,(p,)).fetchall()
                elapsed=(time.perf_counter_ns()-start)/1e6
                assert result==expected[p]
                samples[variant].append(elapsed)
    writes={v:[] for v in dbs}
    for batch in range(10):
        order=list(dbs);rng.shuffle(order)
        extra=[(60000+batch*1000+i,'hot',20000+batch*1000+i,i) for i in range(1000)]
        for v in order:
            start=time.perf_counter_ns()
            dbs[v].executemany('INSERT INTO history VALUES(?,?,?,?)',extra)
            dbs[v].commit()
            writes[v].append((time.perf_counter_ns()-start)/1e6)
        assert dbs['baseline'].execute(sql,('hot',)).fetchall()==dbs['indexed'].execute(sql,('hot',)).fetchall()
    sizes={v:(root/(v+'.sqlite')).stat().st_size for v in dbs}
    for db in dbs.values(): db.close()
    def summary(values):
        return {'n':len(values),'median_ms':statistics.median(values),'p95_nearest_rank_ms':sorted(values)[math.ceil(.95*len(values))-1]}
    return {'rows_initial':60000,'rows_final':70000,'parameters':params,'seed':42,'plans':plans,'read_samples_ms':samples,'read_summary':{v:summary(s) for v,s in samples.items()},'write_1000_rows_samples_ms':writes,'write_summary':{v:summary(s) for v,s in writes.items()},'file_bytes':sizes,'result_equality':True,'conditions':'warm connections, interleaved variant order, rollback journal/default synchronous, one process, no competing load; p95 mixes four query shapes'}

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, help='New JSON report file; parent directory must exist')
    args=parser.parse_args()
    with tempfile.TemporaryDirectory(prefix='engineering-exercise-') as tmp:
        report={'python':platform.python_version(),'sqlite':sqlite3.sqlite_version,'platform':platform.platform(),'source_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'service':verify_service(Path(tmp)),'benchmark':benchmark(Path(tmp))}
    if args.output:
        with args.output.open('x') as stream:
            stream.write(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k!='benchmark'},indent=2))
    print(json.dumps({k:v for k,v in report['benchmark'].items() if 'samples' not in k and k!='plans'},indent=2))
