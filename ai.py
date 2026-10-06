#!/usr/bin/env python3
"""Personal AI harness: global setup, project manifests, catalog and plugin exports."""
import sys

from lib import catalog, harness


def main():
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
