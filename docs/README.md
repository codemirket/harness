# Mirket documentation

Start with [versions and supported targets](release-status.md). These guides describe
Mirket 1.0.11. Use versioned documentation for older binaries and keep optional
container support separate from core CLI platform support.

| Your task | Guide |
| --- | --- |
| Install with an agent | [Agent installation](agent-installation.md) |
| Install manually, update or diagnose an installation | [Installation and updates](installation.md) |
| Choose roles, skills and starter bundles | [Expertise selection](roles.md) |
| Register a project, coordinate work or inspect the MCP tool map | [Workflows and MCP](workflows.md) |
| Verify an artifact or complete a worker handoff | [Task evidence](task-evidence.md) |
| Use local memory, durable workers, retention or recovery | [Agent runtime](agent-runtime.md) |
| Produce and review interfaces, graphics, documents or motion | [Visual production](visual-quality.md) |
| Configure production tools and inspect their output | [Packages](packages.md) · [Podman](podman.md) |
| Understand source ownership and state boundaries | [Architecture](architecture.md) |
| Develop and verify a change | [Contributing](../CONTRIBUTING.md) |
| Inspect source reuse and licenses | [Source review](source-review.md) · [Third-party notices](third-party-notices.md) |

## Keep the checks separate

Setup installs files and configuration. Doctor checks their consistency. A call
inside each desktop observes that live connection. A saved render or test result
shows an operation's output. The host still needs to judge whether the result
satisfies the user's request. No one check replaces the others.
