#!/usr/bin/env python3
"""Personal agent harness: shared capabilities, explicit target adapters."""
import sys

from lib import catalog, harness


def main():
    if len(sys.argv) < 2 or sys.argv[1] in ('-h', '--help'):
        print('''Personal agent harness (Python 3.9+, standard library)

  install [--target codex-desktop|claude-desktop|all] [--dry-run]
                       Install shared guidance, skills, MCP and target settings (default: both desktops)
  check [--target ...]  Read-only installation and configuration drift check
  workbench <command>  Render and check browser, vector, office and Markdown artifacts
  handoff --output DIR Package skills and instructions for Claude Chat/Cowork Customize
  schedule <command>   Plan, install or check the daily midnight OS schedule
  maintenance <command> Pull a clean checkout, sync, or read the last run status
  runtime doctor       Check clients; optionally inspect project tools and capture a local app
  settings <command>   Capture, plan, apply or diagnose portable Codex preferences
  delegate claude      Run a bounded, read-only Claude Code second opinion
  eval <command>       Prepare development tasks, check outputs and record reviews
  plan | sync | doctor Reconcile only global guidance and skills
  project <command>    Initialize, add, plan, sync or diagnose selected project skills
  catalog <command>    Browse and register reviewed skills
  export               Build Codex plugin bundles

Use COMMAND --help. Claude Desktop means local Code mode; Chat/Cowork use account customization.
Schedules, model/appearance preferences and legacy runtime setup are explicit commands.''')
        return 0
    if sys.argv[1] in ('install', 'check'):
        from lib import target_install
        return target_install.main(sys.argv[1:])
    if sys.argv[1] == 'workbench':
        from lib import workbench
        return workbench.main(sys.argv[2:])
    if sys.argv[1] == 'handoff':
        from lib import handoff
        return handoff.main(sys.argv[2:])
    if sys.argv[1] == 'legacy-install':
        from lib import install
        return install.main(['install'] + sys.argv[2:])
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
