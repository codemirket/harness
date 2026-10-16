# Atlassian: access-aware collaboration

**Language:** Access-aware collaboration. This is a curator interpretation of selected relationships, not an upstream category or a claim of product equivalence.

**Use when:** People discuss a shared document while draft state, comment visibility and permission changes must remain explicit.

**Poor fit:** Presence or a mention is being used as authorization, or a real access-control implementation is expected from the local teaching flow.

## Brief → concept

The fictional Mosaic research team is discussing a quiet arrival window. Morgan owns the draft, Jules can edit, and Tessa can view. Sam is present but has no document access. A person must be able to draft a comment, choose its audience, recover after role rejection and understand an owner's explicit permission change.

**Concept:** A discussion is content within an access boundary. Put author, draft state and audience near the work; keep permission facts distinguishable from social cues. Preserve unposted text when an action fails.

## Source evidence → transferable grammar

**Actually observed:** Atlassian's live Comment component page was rendered at 1360×1000 on 2026-10-15; the default author/content hierarchy was visible. Its documented restricted and saving variants were read. Confluence Cloud's restrictions guidance was also rendered/read, while sharing and draft guidance were read. These are component/support sources, not an authenticated Confluence sharing session. Current support behavior can differ by plan and content type; page drafts are not live docs.

**Grammar to transfer:** Group author and audience with a comment, show meaningful pending/rejected state, make actual named access inspectable, and preserve a recoverable draft. The sharing guidance distinguishes a copied link from a grant. These sources do not make presence, an avatar or a mention an authorization decision. This specimen invents a small explicit local role model rather than copying Confluence's permission hierarchy.

## Located decisions in the original specimen

| Location in `index.html` | Decision | Purpose |
| --- | --- | --- |
| `.presence`, `#access-summary`, `#access-list` | Present people and named access are displayed separately; Sam initially has none. | A social cue cannot silently broaden access. |
| `#comment-draft`, `#comment-scope`, `#composer-scope` | Text and its current audience remain together. Mentions are literal text and do not add grants. | The person can inspect who will see the comment before posting. |
| `#comment-status`, `#closed-draft`, `#resume-draft` | Viewer posting is rejected; Close/Resume keeps text and audience on the page, with focus restored. | Failure and interruption do not erase work or imply publication. |
| `#share-form`, `#access-status`, `#local-role` | An editor/viewer cannot change access; an owner grant updates the list and audience count. | Permission feedback corresponds to an actual local state change. |
| `#comments`, `#comment-count` | Successful comments append once as text nodes and name their audience. Viewer simulation hides owner/editor comments. | User text remains text; no fabricated notification or backend effect is implied. |

All identity, content, CSS and JavaScript are original; avatars use fictional initials and system fonts. The note, existing comment and named access facts remain readable with JavaScript off. Enhanced posting and grants are unavailable. The teaching aside is outside `main`. Local role selection is a simulation, not authentication or a security boundary.

## Attractive wrong variant

Open `?variant=misleading`. A generous, polished collaboration banner claims every present person can read the draft, including Sam. The actual access list still excludes Sam. The banner uses presence as proof of permission. The wrong variant retains the working role rejection and local grant behavior so the false inference can be checked against the same underlying facts.

## Discriminating failure probe

Write a comment mentioning Sam, choose “Owner and editors only”, switch the local role to Viewer, and post. The count must remain **1**, rejection must be explicit, and both text and audience must survive. Close and Resume must restore the draft and focus. Attempt to give Sam Editor access as Viewer: membership must stay **3**. Switch to Owner and retry: membership becomes **4**, and the editors audience becomes Morgan, Jules and Sam. Post once; count becomes **2**. Switching to Viewer must hide that restricted comment and show **1** visible comment; switching back to Owner restores **2**. Mentioning Sam alone must never grant access.

Use `#comment-draft`, `#comment-scope`, `#composer-scope`, `#comment-status`, `#access-list`, `#access-status` and their matching `data-testid` values to inspect effects. Reject the misleading banner when presence and actual grants disagree. Inspect 390px, enlarged text and visible focus; no required action is hover-only.

## Materially different transfer task

Design a sensitive case-handover discussion with expiring reviewer access and field-level masking. Specify which case fields a temporary reviewer can read, what happens if their permission expires while a draft is open, and how a rejected post preserves the text without revealing restricted fields. Produce the current-access, expired-access and recovered-draft states. Do not reuse Mosaic's roles, member counts or assumption that document access and every field's visibility are identical.

**Reject the transfer when:** A presence marker overrides expired access, or recovering a draft reveals information the viewer is no longer allowed to read. The production system must enforce its real contract beyond the UI.

## Exact owner, read and render route

Craft owner: `interface-design`. Embedded owner: `design-inspiration`. Instance ID: `atlassian-collaboration`.

```sh
mirket catalog read design-inspiration --path references/design-languages/atlassian-collaboration.md
mirket catalog read design-inspiration --path assets/design-languages/atlassian-collaboration/index.html
```

MCP: `skill_read {"skill_id":"design-inspiration","path":"references/design-languages/atlassian-collaboration.md"}` and `skill_read {"skill_id":"design-inspiration","path":"assets/design-languages/atlassian-collaboration/index.html"}`.

Resources: `mirket://skills/design-inspiration/references/design-languages/atlassian-collaboration.md` and `mirket://skills/design-inspiration/assets/design-languages/atlassian-collaboration/index.html`.

Render the saved HTML in the actual installed `design-inspiration` directory with a local-file browser open or loopback server. No build/install is needed. Project copies commonly use `.agents/skills/design-inspiration/` for Codex or `.claude/skills/design-inspiration/` for Claude; use the route returned by Mirket. Compare the base and `?variant=misleading`; reading the source does not render it.

## Source edition, rights and exclusions

- [Atlassian Design System: Comment examples](https://atlassian.design/components/comment/examples) — Mutable live component documentation inspected 2026-10-15. [ADS license](https://atlassian.design/license), last updated 2025-03-13, has restricted terms; it is not a blanket open-source grant for its documentation, code, visual assets or branding. No ADS material is redistributed.
- [Confluence Cloud: Change who can find content and what they can do](https://support.atlassian.com/confluence-cloud/docs/add-or-remove-page-restrictions/) — Current restrictions documentation rendered/read 2026-10-15. Named/general access guidance; no authenticated product flow tested.
- [Confluence Cloud: Share content](https://support.atlassian.com/confluence-cloud/docs/share-confluence-content/) — Current sharing guidance read 2026-10-15. Link sharing and permissions have different effects. No sharing action or notification sent.
- [Confluence Cloud: Create, update, and manage written content](https://support.atlassian.com/confluence-cloud/docs/create-edit-and-publish-a-page/) — Current draft guidance read 2026-10-15. Product autosave and real-time collaboration were not tested or imported.

All references are **link-only**. Support documentation/media rights are separate from software licenses. No Atlassian component code, tokens, icons, fonts, screenshots, claims or brand identity are included. The original example uses the repository's own license.

## Verification scope

On 2026-10-15 local Chrome 155 on macOS rendered both variants at 1280×900 and 390×900. Checks cover native focus, text at 200%, JavaScript-off essential reading, zero external requests and axe purposeful-default checks. Exercised effects include viewer rejection, audience/draft preservation, Close/Resume focus, rejected and successful local grants, resulting audience counts, one comment per nonempty post, literal rendering of markup-like text, and role-dependent hiding/restoring of restricted comments. No real authorization enforcement, invitation, notification, durable autosave, simultaneous editing, screen-reader session, physical device or human visual acceptance is claimed.
