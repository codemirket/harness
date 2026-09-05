# Custom Skills

This directory owns custom skills you create and maintain. Keep re-installable
third-party skills, including Windmill skills, in their external installers or
plugin-managed locations.

No custom skills are included yet. Add each skill at:

```text
skills/<group>/<skill-name>/SKILL.md
```

Use a skill name unique across groups, since agent skill directories are flat.
Keep any supporting scripts, references, templates, or assets inside the skill's
directory. Directories without `SKILL.md` are not installed.

Run the appropriate scripts under `setup/` after adding, removing, or renaming a
skill, then restart the affected agents. Setup links each custom skill individually
and removes stale skill links pointing into this shared repository. An empty
collection is valid.
