# Worked recipe: add a service package without replacing a monorepo

Scenario: the repo already has a root lockfile, workspace manifest, CI and a
modified application. The requested stack is explicit. The goal is one new
service package, not reinitializing the root.

1. Capture `git status --short` and inspect the destination and workspace
   registration. Check the root's supported package manager and runtime; use
   its existing mechanism rather than adding competing lockfiles.
2. Locate the installed native generator through the configured toolchain.
   Read that exact version's help and relevant official documentation. Identify
   whether generation invokes package installation, network downloads, lifecycle
   scripts or Git initialization. A package-runner command may download even
   when its name looks local.
3. Choose a new empty staging directory outside existing source paths. Run only
   supported generation options for the selected template with side effects
   disabled. If no installed generator or no non-installing mode exists, report
   that concrete dependency. You may prepare integration decisions and a minimal
   requested file edit without pretending a native generation occurred.
4. Inventory generated files. Read entry point, dependency scripts and config;
   remove template-only content within staging. Do not copy `.git`, caches,
   secrets or generated dependencies. Compare each destination path; copy new
   files, merge shared configuration deliberately and preserve unrelated work.
5. Register the package using the repository convention. Add the smallest
   representative route/command. Test valid input, rejected input and actual
   execution through the generated entry point using existing dependencies.
6. Inspect the final diff and status against the initial state. Root files must
   change only where integration required it. Report actual commands and gates;
   “scaffold complete, dependencies unavailable” differs from “service runs.”

A useful regression for the scaffold procedure: create a disposable fixture
with a sentinel root config and an uncommitted file, stage the generated package,
then confirm both originals remain byte-identical and the new entry point runs.
Test conflict handling using an existing destination file. This verifies merge
safety; it does not validate every template supplied by the framework.
