On macOS, Codex finds personal skills in **`~/.agents/skills`**, which is `/Users/<your-username>/.agents/skills`.

**Official source:** https://learn.chatgpt.com/docs/build-skills#where-codex-loads-local-skills

I checked this with the OpenAI docs MCP server by searching and then fetching that section. Its table of skill locations lists the `USER` scope as `$HOME/.agents/skills`, described as "Any skills checked into the user's personal folder," for skills you want in any repository you work in.

- **macOS path is my inference:** the docs give the same `$HOME` path for every platform and don't mention macOS. On macOS `$HOME` is `/Users/<username>`, which is how I got the path above.
- **Other locations on that page:** repository skills go in `.agents/skills`, checked in every folder from the current one up to the repo root. Machine-wide admin skills go in `/etc/codex/skills`. Codex follows symlinked skill folders.
- **Older path:** a second search for `~/.codex/skills` found no official page using that path.

I didn't edit any files or change any settings.
