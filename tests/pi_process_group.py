"""Bounded cleanup for process groups created by disposable Pi fixtures."""

import os
import signal
import subprocess
import time


def active_group_members(pgid):
    output = subprocess.check_output(
        ["/bin/ps", "-axo", "pid=,pgid=,stat=,command="], text=True
    )
    members = []
    for line in output.splitlines():
        parts = line.split(None, 3)
        if len(parts) == 4 and int(parts[1]) == pgid and not parts[2].startswith("Z"):
            members.append({"pid": int(parts[0]), "stat": parts[2], "command": parts[3]})
    return members


def terminate_owned_group(process, markers, grace_seconds=1.0):
    """Stop a fixture's group even when its leader exited; refuse an unowned group."""
    pgid = process.pid  # Popen(start_new_session=True) makes the leader its group ID.
    before = active_group_members(pgid)
    leader_is_member = process.poll() is None and any(member["pid"] == pgid for member in before)
    marked_descendant = any(
        any(marker in member["command"] for marker in markers) for member in before
    )
    if before and not (leader_is_member or marked_descendant):
        return {"before": before, "after": before, "refused_unowned_group": True}
    if before:
        try:
            os.killpg(pgid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        deadline = time.monotonic() + grace_seconds
        while time.monotonic() < deadline and active_group_members(pgid):
            time.sleep(0.05)
        if active_group_members(pgid):
            try:
                os.killpg(pgid, signal.SIGKILL)
            except ProcessLookupError:
                pass
    if process.poll() is None:
        try:
            process.wait(timeout=2)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=2)
    after = active_group_members(pgid)
    return {"before": before, "after": after, "refused_unowned_group": False}
