"""Synthetic local scaffold, queue and plan-input exercises, not infrastructure tooling.

Uses only Python standard library and temporary files. No native generator, cloud
provider, worker deployment or production capacity is exercised. No external I/O.
Optional --output stores a NEW JSON report; existing reports are never replaced.
"""
import argparse
import hashlib
import json
from pathlib import Path
import queue
import subprocess
import sys
import tempfile


def snapshot(root):
    return {str(p.relative_to(root)):p.read_bytes() for p in root.rglob('*') if p.is_file()}


def merge_fixture(stage, target):
    """Fixture-only all-path preflight, then merge; not a transactional filesystem API."""
    sources=list(stage.rglob('*'))
    if any(p.is_symlink() for p in sources):
        raise ValueError('linked staged source')
    files=[p for p in sources if p.is_file()]
    for source in files:
        relative=source.relative_to(stage)
        destination=target/relative
        # Reject file conflicts and incompatible/linked ancestors before first write.
        if destination.exists() or destination.is_symlink():
            raise ValueError('destination conflict')
        for ancestor in destination.parents:
            if ancestor==target:
                break
            if ancestor.is_symlink() or (ancestor.exists() and not ancestor.is_dir()):
                raise ValueError('destination ancestor conflict')
    for source in files:
        destination=target/source.relative_to(stage)
        destination.parent.mkdir(parents=True,exist_ok=True)
        with destination.open('xb') as stream:
            stream.write(source.read_bytes())


def scaffold(root):
    target=root/'existing'; target.mkdir()
    (target/'workspace.json').write_text('{"packages":[],"keep":true}\n')
    (target/'uncommitted.txt').write_text('user draft stays unchanged\n')
    original=snapshot(target)
    stage=root/'stage'; stage.mkdir()
    (stage/'service').mkdir()
    (stage/'service/main.py').write_text('import argparse\np=argparse.ArgumentParser()\np.add_argument("count",type=int)\na=p.parse_args()\nif a.count<0:p.error("count must be nonnegative")\nprint(a.count*2)\n')
    merge_fixture(stage,target)
    assert all(snapshot(target)[p]==value for p,value in original.items())
    good=subprocess.run([sys.executable,str(target/'service/main.py'),'7'],capture_output=True,text=True,timeout=5)
    bad=subprocess.run([sys.executable,str(target/'service/main.py'),'-1'],capture_output=True,text=True,timeout=5)
    assert good.returncode==0 and good.stdout.strip()=='14'
    assert bad.returncode==2 and 'count must be nonnegative' in bad.stderr
    prior=snapshot(target)
    (stage/'fresh.py').write_text('must not be copied on conflict\n')
    try:
        merge_fixture(stage,target)
    except ValueError as error:
        assert str(error)=='destination conflict'
    else:
        raise AssertionError('expected conflict')
    assert snapshot(target)==prior
    # A linked destination ancestor must not escape the named fixture root.
    links=root/'linked-stage'; (links/'alias').mkdir(parents=True)
    (links/'alias/new.txt').write_text('escape probe')
    outside=root/'outside'; outside.mkdir()
    linked_check = 'passed'
    try:
        (target/'alias').symlink_to(outside,target_is_directory=True)
    except (OSError, NotImplementedError):
        linked_check = 'skipped: host symlink creation unavailable'
    if linked_check == 'passed':
        try:
            merge_fixture(links,target)
        except ValueError as error:
            assert str(error)=='destination ancestor conflict'
        else:
            raise AssertionError('expected linked ancestor rejection')
    assert list(outside.iterdir())==[]
    return {'checks':5 if linked_check == 'passed' else 4,'preserved_original_files':len(original),'valid_entry_stdout':good.stdout.strip(),'invalid_input_exit':bad.returncode,'conflict_rejected_without_partial_copy':True,'linked_ancestor_rejected':linked_check == 'passed','linked_ancestor_probe':linked_check,'native_generator_used':False}


def overload():
    # Deterministic ticks model an unavailable dependency followed by recovery.
    jobs=queue.Queue(maxsize=3)
    accepted=[]; rejected=[]; completed=[]; trace=[]
    for tick in range(12):
        available=tick>=3
        if available:
            try:
                completed.append(jobs.get_nowait());jobs.task_done()
            except queue.Empty:
                pass
        arriving=[tick*2,tick*2+1] if tick<6 else []
        for job in arriving:
            try:
                jobs.put_nowait(job);accepted.append(job)
            except queue.Full:
                rejected.append(job)
        assert jobs.qsize()<=3
        assert len(accepted)==len(completed)+jobs.qsize()
        assert len(set(completed))==len(completed)
        assert not set(rejected)&set(accepted)
        trace.append({'tick':tick,'dependency_available':available,'queue':jobs.qsize(),'completed':len(completed),'rejected':len(rejected)})
    assert rejected and jobs.empty() and set(completed)==set(accepted)
    assert len(accepted)+len(rejected)==12
    return {'checks':6,'queue_limit':3,'offered':12,'accepted':len(accepted),'rejected':len(rejected),'completed':len(completed),'trace':trace,'limits':'Discrete local model, no real queue broker, crash durability or measured throughput'}


def fingerprint(config,state,target):
    value={'config':config,'state':state,'target':target}
    return hashlib.sha256(json.dumps(value,sort_keys=True,separators=(',',':')).encode()).hexdigest()


def stale_plan():
    desired={'worker_replicas':2,'public_ingress':False}
    observed={'generation':7,'worker_replicas':1}
    target={'account':'fixture','environment':'local'}
    reviewed=fingerprint(desired,observed,target)
    assert fingerprint(desired,observed,target)==reviewed
    changes=[(dict(desired,public_ingress=True),observed,target),
             (desired,dict(observed,generation=8),target),
             (desired,observed,dict(target,environment='other'))]
    rejected=[fingerprint(*args)!=reviewed for args in changes]
    assert all(rejected)
    return {'checks':4,'unchanged_inputs_match':True,'changed_config_state_target_rejected':rejected,'limits':'Synthetic input identity check; no Terraform plan/apply, provider state refresh or runtime health verification'}


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,help='New JSON report file; parent must exist')
    args=parser.parse_args()
    with tempfile.TemporaryDirectory(prefix='systems-assurance-') as temp:
        report={'source_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'scaffold':scaffold(Path(temp)),'queue':overload(),'plan_inputs':stale_plan()}
    rendered=json.dumps(report,indent=2)+'\n'
    if args.output:
        with args.output.open('x') as stream:
            stream.write(rendered)
    print(rendered,end='')
