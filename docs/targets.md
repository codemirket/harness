# Target adapters and coverage

The harness keeps shared content in `instructions/`, `skills/` and
`registry/harness.json`. `registry/targets.json` owns client destinations and
settings; `registry/mcp.json` describes servers independently of client syntax.
Target adapters render these sources into native configuration.

## Installed surfaces

| Public target | Shared instructions | Global skills | Settings and MCP |
| --- | --- | --- | --- |
| `codex-desktop` | `~/.codex/AGENTS.md` | `~/.agents/skills/` | Codex `config.toml`, `mcp_servers` |
| `claude-desktop` | `~/.claude/CLAUDE.md` | `~/.claude/skills/` | Claude `settings.json`, Code MCP in `~/.claude.json` |

Paths above show the standard home-relative layout. Custom `CODEX_HOME` or
`CLAUDE_CONFIG_DIR` roots are rejected when they point elsewhere; the harness does
not silently configure a different client. The installer's `--home` prepares an
alternate home for inspection; it does not reconfigure a running application's
home. The default `all` and legacy `both` select the two desktop targets.
Codex Desktop shares these files with Codex CLI; Claude Desktop support here means
local **Code** sessions and Claude Code CLI. See
[Claude Desktop shared configuration](https://code.claude.com/docs/en/desktop#shared-configuration).

The default policies are Codex `on-request` approval with `workspace-write`
sandboxing and Claude's `default` permission mode. OpenAI documentation MCP is
enabled; Microsoft Learn is present but disabled. No credentials are supplied.

MCP `enabled: false` excludes a source from installation; it does not delete an
already configured server. Removed skills, servers and settings are preserved for
deliberate cleanup. Existing server identities must match before configuration can
merge, so credentials cannot silently follow a changed URL or command.

## Claude Desktop Code, Chat and Cowork

Local Code sessions share Claude Code's user configuration. Personal skills take
precedence over project skills with the same name. Keep global and
project names distinct unless the override is deliberate.
[Claude skills](https://code.claude.com/docs/en/skills).

Chat and Cowork are separate surfaces. Cowork uses skills enabled through the
account's Customize interface, rather than the host's `~/.claude/skills` directory.
The local Code installation does not install account skills.
[Desktop customization](https://code.claude.com/docs/en/desktop#extend-claude-code).

```sh
mkdir -p build
python3 ai.py handoff --output build/claude-account-handoff
```

This creates a new directory containing instructions, one ZIP per portable global
skill, a neutral remote connector list, and a local stdio MCP fragment for Chat.
The local `skill-catalog` installer is excluded from account skill packages.
Review and upload skills through the client, and configure connectors manually.
Generation does not upload, enable, authenticate or invoke anything.

Chat's local MCP configuration on macOS is
`~/Library/Application Support/Claude/claude_desktop_config.json`; Windows uses
`%APPDATA%\Claude\claude_desktop_config.json`. Local Code also loads this file;
duplicate names can override Code MCP definitions. Legacy Chat JSON configuration
must not be treated as universal Cowork support.
[Local MCP setup](https://modelcontextprotocol.io/docs/develop/connect-local-servers),
[Desktop MCP behavior](https://code.claude.com/docs/en/desktop#mcp-servers-from-the-claude-desktop-chat-app).

## What enforcement means

Instructions and skills supply guidance. Native settings can constrain supported
actions; they are not a policy engine shared across clients. User configuration
remains editable, project settings can affect behavior, and existing per-tool
allow rules remain intact. The harness's default confirmation settings therefore
do not establish complete lockdown. Claude explicitly distinguishes instructional
context from enforced settings. [Claude instructions](https://code.claude.com/docs/en/memory).

`check` verifies installed content and selected configuration, not authentication,
model access, skill invocation or live MCP health. Test those
in the client after installation. Remote servers can require account setup, and
stdio servers require their declared executable and dependencies.

Official target behavior above was reviewed on 2026-10-09. Installed client versions
can differ; use their visible discovery and permission behavior as the final check.

## Removed target

Zed is no longer an installation or project target. Its existing settings and
extensions are left in place. The harness does not uninstall previous
configurations or remove shared `.agents/skills` files needed by Codex.
