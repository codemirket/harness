"""Install the declared Codex desktop harness after read-only prerequisite checks.

Third-party applications, authentication, project skills and plugin packages are
never installed by this command. Their availability is reported separately.
"""
import argparse
import json
import os
from pathlib import Path
import sys

from . import harness, runtime, schedule, settings


def preflight(*, home=None, mode='auto', codex=None, claude=None, desktop=None,
              settings_path=None, register_schedule=True):
    report = {'schema_version': 1, 'primary_client': 'codex-desktop',
              'status': 'planned', 'applied': False, 'blockers': [],
              'global_skills': [], 'settings': None, 'schedule': None}
    if home is None and os.environ.get('CODEX_HOME') and settings.codex_home().resolve() != (Path.home() / '.codex').resolve():
        report['blockers'].append('Complete installation requires the standard ~/.codex home. CODEX_HOME points elsewhere; use individual settings/sync commands and deliberately arrange guidance for that client.')
    if register_schedule and settings_path is not None and Path(settings_path).resolve() != (harness.ROOT / 'registry/codex-settings.json').resolve():
        report['blockers'].append('Daily maintenance uses the repository settings source; use --no-schedule with an alternative settings manifest.')
    report['runtime'] = runtime.diagnose(home=home, codex=codex, claude=claude,
                                         desktop=desktop, check_auth=True)
    if not report['runtime']['ready']:
        report['blockers'].append('Required runtimes are missing or inaccessible; follow runtime guidance.')
    try:
        _, _, _, jobs = harness.global_plan('both', home, mode)
        report['global_skills'] = harness.public_jobs(jobs)
    except (ValueError, OSError, KeyError) as error:
        report['blockers'].append('Global setup: ' + str(error))
    try:
        report['settings'] = settings.plan(home, settings_path)
        if not report['settings']['ready_to_apply']:
            report['blockers'].append(report['settings']['instruction'])
    except (ValueError, OSError, KeyError) as error:
        report['blockers'].append('Portable settings: ' + str(error))
    if register_schedule:
        try:
            report['schedule'] = schedule.plan(root=harness.ROOT, home=home, mode=mode)
            if not report['schedule']['ready_to_apply']:
                report['blockers'].extend('Daily schedule: ' + reason for reason in report['schedule']['blockers'])
        except (ValueError, OSError, KeyError) as error:
            report['blockers'].append('Daily schedule: ' + str(error))
    report['authentication_attention'] = report['runtime']['authentication_attention']
    report['ready_to_install'] = not report['blockers']
    report['changes_pending'] = (any(job['action'] != 'unchanged' for job in report['global_skills'])
                                  or bool(report['settings'] and report['settings']['changed'])
                                  or bool(report['schedule'] and report['schedule']['changed']))
    report['ready'] = bool(report['ready_to_install'] and not report['changes_pending']
                           and not report['authentication_attention'])
    if report['blockers']:
        report['status'] = 'blocked'
    elif report['ready']:
        report['status'] = 'ready'
    elif report['authentication_attention']:
        report['status'] = 'authentication_attention'
    report['limits'] = [
        'Authentication status does not verify model entitlement, quota or a live model response.',
        'The harness does not install plugins, connect accounts or grant permissions. Codex may fetch configured plugins on marketplace refresh.',
        'Global items have individual recovery; settings have a local backup. This is not a whole-install transaction.',
        'The daily OS job pulls only a clean checkout and defers changed preferences while Codex is open.',
    ]
    return report


def install(*, dry_run=False, **options):
    result = preflight(**options)
    if dry_run or not result['ready_to_install']:
        return result
    # Reconciliation and settings each repeat their own concurrency checks.
    # Known conflicts were checked before either component could mutate files.
    try:
        result['global_skills'] = harness.sync_global('both', options.get('home'), options.get('mode', 'auto'))
    except (ValueError, OSError, KeyError) as error:
        result.update(status='partially_applied', ready=False, applied=None)
        result['blockers'].append('Global installation stopped and may have completed some items; settings were not applied. Inspect the ownership receipt, fix the cause and rerun: ' + str(error))
        return result
    result['applied'] = True
    try:
        result['settings'] = settings.apply(options.get('home'), options.get('settings_path'))
    except (ValueError, OSError, KeyError) as error:
        result.update(status='partially_applied', ready=False)
        result['blockers'].append('Global setup completed; portable settings were not completed: ' + str(error))
        return result
    if result['settings'].get('app_must_close'):
        result.update(status='partially_applied', ready=False)
        result['blockers'].append('Global setup completed; ' + result['settings']['instruction'])
        return result
    if options.get('register_schedule', True):
        try:
            result['schedule'] = schedule.apply(root=harness.ROOT, home=options.get('home'), mode=options.get('mode', 'auto'))
            if not result['schedule'].get('verified'):
                result.update(status='partially_applied', ready=False)
                result['blockers'].append('Files are synchronized; daily OS schedule registration was not verified. Inspect the schedule report and rerun.')
                return result
        except (ValueError, OSError, KeyError) as error:
            result.update(status='partially_applied', ready=False)
            result['blockers'].append('Files are synchronized; daily OS schedule registration failed: ' + str(error))
            return result
    result['changes_pending'] = False
    result['ready'] = not result['authentication_attention']
    result['status'] = 'installed' if result['ready'] else 'installed_authentication_attention'
    return result


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['install', 'check'])
    parser.add_argument('--dry-run', action='store_true', help='Run all preflight checks without harness writes')
    parser.add_argument('--home', type=Path, help='Existing target user home (does not redirect CLI authentication)')
    parser.add_argument('--mode', choices=['auto', 'link', 'copy'], default='auto')
    parser.add_argument('--no-schedule', dest='register_schedule', action='store_false',
                        help='Explicitly skip OS schedule setup/check (for isolated homes or exported bundles)')
    for name in ('codex', 'claude', 'desktop'):
        parser.add_argument('--' + name, type=Path, help='Explicit runtime path for a custom installation')
    parser.add_argument('--settings', dest='settings_path', type=Path, help='Alternative portable preferences manifest')
    args = vars(parser.parse_args(argv))
    command = args.pop('command')
    dry_run = args.pop('dry_run')
    result = preflight(**args) if command == 'check' else install(dry_run=dry_run, **args)
    print(json.dumps(result, indent=2))
    if command == 'install' and dry_run:
        return 0 if result['ready_to_install'] and not result['authentication_attention'] else 1
    return 0 if result['ready'] else 1


if __name__ == '__main__':
    sys.exit(main())
