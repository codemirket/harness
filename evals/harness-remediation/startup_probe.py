#!/usr/bin/env python3
"""Bounded, disposable MCP startup samples. No model inference or installer."""
import argparse
import asyncio
import hashlib
import json
import os
from pathlib import Path
import platform
import tempfile
import time


async def sample(binary, slot):
    with tempfile.TemporaryDirectory(prefix="mirket-startup-") as home:
        process = await asyncio.create_subprocess_exec(
            str(binary), "--home", home, "mcp", "serve",
            stdin=asyncio.subprocess.PIPE, stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
            env=dict(os.environ, MIRKET_STARTUP_TIMING="1"),
        )
        started = time.monotonic()
        request = {
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {"protocolVersion": "2025-03-26", "capabilities": {},
                       "clientInfo": {"name": "startup-probe", "version": "1"}},
        }
        error = None
        try:
            process.stdin.write((json.dumps(request) + "\n").encode())
            await process.stdin.drain()
            reply = json.loads(await asyncio.wait_for(process.stdout.readline(), 15))
            if reply.get("id") != 1 or "result" not in reply:
                raise ValueError("invalid initialization response")
        except (asyncio.TimeoutError, ValueError, OSError) as failure:
            error = type(failure).__name__
        elapsed = (time.monotonic() - started) * 1000
        if error is None:
            process.stdin.write(
                b'{"jsonrpc":"2.0","method":"notifications/initialized"}\n'
                b'{"jsonrpc":"2.0","id":2,"method":"ping"}\n'
            )
            try:
                await process.stdin.drain()
                pong = json.loads(await asyncio.wait_for(process.stdout.readline(), 5))
                if pong.get("id") != 2 or "result" not in pong:
                    error = "invalid_ping_response"
            except (asyncio.TimeoutError, ValueError, OSError) as failure:
                error = type(failure).__name__
        if process.returncode is None:
            process.terminate()
        try:
            _, stderr = await asyncio.wait_for(process.communicate(), 5)
        except asyncio.TimeoutError:
            process.kill()
            _, stderr = await process.communicate()
        phases = []
        for line in stderr.decode(errors="replace").splitlines():
            try:
                value = json.loads(line)
                if "mirket_startup" in value:
                    phases.append(value)
            except (ValueError, TypeError):
                pass
        return {"slot": slot, "ok": error is None, "initialize_ms": elapsed,
                "error": error, "phases": phases}


async def run(args):
    binary = args.binary.resolve(strict=True)
    before = hashlib.sha256(binary.read_bytes()).hexdigest()
    result = {"schema": 1, "binary_sha256": before, "platform": platform.platform(),
              "deadline_seconds": 15, "groups": []}
    for count in args.concurrency:
        rows = await asyncio.gather(*(sample(binary, i) for i in range(count)))
        result["groups"].append({"concurrency": count, "samples": rows})
    result["binary_unchanged"] = before == hashlib.sha256(binary.read_bytes()).hexdigest()
    result["ok"] = result["binary_unchanged"] and all(
        row["ok"] for group in result["groups"] for row in group["samples"]
    )
    with args.output.open("x") as output:
        json.dump(result, output, indent=2)
        output.write("\n")
    print(json.dumps({"ok": result["ok"], "output": str(args.output)}))
    return 0 if result["ok"] else 1


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--concurrency", nargs="+", type=int, default=[1, 4, 8])
    arguments = parser.parse_args()
    if not 1 <= len(arguments.concurrency) <= 4 or any(
        not 1 <= count <= 16 for count in arguments.concurrency
    ):
        parser.error("choose one to four concurrency levels, each from 1 to 16")
    if arguments.output.exists():
        parser.error("output already exists; prior evidence is never overwritten")
    raise SystemExit(asyncio.run(run(arguments)))
