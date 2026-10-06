#!/usr/bin/env python3
"""Codex desktop harness with portable settings and selective Claude CLI delegation."""
import sys

from lib import catalog, harness


def main():
    if len(sys.argv) < 2 or sys.argv[1] in ('-h', '--help'):
        print('''Personal Codex desktop harness (Python 3.9+)

  install [--dry-run]   Verify runtimes; install guidance, skills, settings and daily OS job
  check                Read-only check of runtimes, global setup, settings and OS job
  schedule <command>   Plan, install or check the daily midnight OS schedule
  maintenance <command> Pull a clean checkout, sync, or read the last run status
  runtime doctor       Discover Codex desktop, Codex CLI and Claude Code CLI
  settings <command>   Capture, plan, apply or diagnose portable Codex preferences
  delegate claude      Run a bounded, read-only Claude Code second opinion
  eval <command>       Prepare development tasks, check outputs and record reviews
  plan | sync | doctor Reconcile global guidance and skills (default: Codex)
  project <command>    Initialize, add, plan, sync or diagnose selected project skills
  catalog <command>    Browse and register reviewed skills
  export               Build Codex plugin bundles

Use COMMAND --help for its arguments. Claude desktop is not supported.''')
        return 0
    if sys.argv[1] in ('install', 'check'):
        from lib import install
        return install.main(sys.argv[1:])
    if sys.argv[1] == 'schedule':
        from lib import schedule
        return schedule.main(sys.argv[2:])
    if sys.argv[1] == 'maintenance':
        from lib import maintenance
        return maintenance.main(sys.argv[2:])
    if sys.argv[1] == 'runtime':
        from lib import runtime
        return runtime.main(sys.argv[2:])
    if sys.argv[1] == 'settings':
        from lib import settings
        return settings.main(sys.argv[2:])
    if sys.argv[1] == 'eval':
        from lib import evaluation
        return evaluation.main(sys.argv[2:])
    if sys.argv[1] == 'delegate':
        if len(sys.argv) < 3 or sys.argv[2] != 'claude':
            raise ValueError('Use: ai.py delegate claude --help')
        from lib import claude_delegate
        return claude_delegate.main(sys.argv[3:])
    if len(sys.argv) > 1 and sys.argv[1] == 'catalog':
        del sys.argv[1]
        catalog.main()
        return 0
    if len(sys.argv) > 1 and sys.argv[1] == 'export':
        from lib import bundle
        return bundle.main(sys.argv[2:])
    return harness.main()


if __name__ == '__main__':
    try:
        sys.exit(main())
    except (ValueError, OSError, KeyError) as error:
        print('Error: ' + str(error), file=sys.stderr)
        sys.exit(1)
