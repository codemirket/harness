# Duolingo case studies: character response

**Character-led finite response** is a Mirket interpretation of dated Duolingo design case studies. The transferable idea is an authored reaction to a true event, not a generic “fun” preset or a current Duolingo application specification.

## Brief → concept → grammar

**Brief.** A small field-learning exercise should acknowledge a correct identification, explain an incorrect one and let the learner try again immediately. The character cannot invent progress.

**Concept.** Pip, an original seed courier, notices the learner’s result. A correct answer earns one small performance; an error gets a thoughtful response and correction. The notebook remains the authority for the actual result.

**Grammar.** Stage one focal event. A slight gathering precedes a hop; a flexible tuft follows the body, then everything rests. The character’s identity, truthful outcome and usable next action remain invariant. Motion reinforces the result while visible text communicates it independently.

## Located choices in the original study

| Location | Choice and purpose |
| --- | --- |
| `#character-stage`, `#actor`, `#tuft`, `#arm` | A terracotta seed silhouette, teal tuft and attached arm make an original cast. Named nested groups keep placement separate from movement; body, arm and tuft have distinct transform origins. |
| Correct-answer performance | Gathering → hop → arm response → tuft follow-through lasts under one second and ends in a composed pose. These are original teaching timings and geometry, not extracted Duolingo keyframes. |
| Incorrect-answer response | A contained lean supports a thinking pose. No leap or success act contradicts the correction. |
| `#answer-status`, `#progress`, `#specimen[data-count]` | The factual outcome and count commit at activation. Repeated correct input does not invent additional completed observations. The next answer is usable during playback. |
| Replay response / Start again | Replay performs the existing event without changing progress. Start again cancels the performance and restores the question. No ambient or perpetual character loop runs. |

**Fit:** bounded learning feedback, companions and meaningful achievements with a defensible character concept. **Poor fit:** every routine action, an urgent error needing immediate corrective work, or a brief with no reason for a character. An appealing performance does not establish motivation or audience response without evidence.

## Contrast and failure probe

`?variant=misleading` uses the same finished character and performance but celebrates Willow, the wrong answer. The correction and zero count remain truthful; the character contradicts them. This is a causal design error, not an intentionally crude drawing.

Probe: choose Willow. In the purposeful route the event is `incorrect`, count is `0`, and `#character-stage[data-reaction="think"]` agrees with the correction. The misleading route instead has `data-reaction="celebrate"`; it must fail even if playback is smooth. Then choose Maple → Willow → Maple → Willow during motion. The last event, count, correction and final resting reaction must belong to Willow. Changing the motion preference mid-hop must cancel travel while preserving that result. Replay must never increment count.

## Transfer task

Create a different companion for a fictional recycling lesson. Give it a silhouette, material and flexible part that explain its own movement. Perform only for a confirmed correct sorting event, retain a useful incorrect-answer response and rest after each. Check the outcome/performance relationship during rapid alternating answers. Copying Pip’s geometry, recoloring Duo or adding a generic bounce does not demonstrate transfer.

## Source, edition, observation and exclusions

[You’re on fire! Or, how we brought the streak milestone to life](https://blog.duolingo.com/streak-milestone-design-animation/) is a January 21, 2022 case study with rough/final Phoenix Duo transformation examples. [Building character](https://blog.duolingo.com/building-character/) describes individual correct-answer performances, including Junior. Checked 2026-10-15. The prior research read both official case studies and sampled altered rough/final GIF poses in the streak article; it did not inspect the entire loop or Junior’s full playback. The current character article text was read again. These historical examples support event/acting/rest as a proposed transfer; they do not establish a 2026 app frequency, reduced-motion or authoring-runtime specification.

The official blog states all rights reserved. Duo, Phoenix Duo, Junior, the remaining cast, marks, GIFs and illustrations are excluded and link-only (`license_url: null`). Pip, the leaf, interface, copy and animation are original. No traced brand artwork, source video, character pack, external font or upstream code is bundled.

## Read, render and build owner

- Study owner: `design-inspiration`. Build owner: `motion-design` for interaction causality; `svg-animation` and `references/svg-motion-craft.md` own the editable vector performance. Use `advanced-motion`’s art-direction reference when the actual brief needs a larger narrative.
- Card resource: `mirket://skills/design-inspiration/references/design-languages/duolingo-performance.md`.
- CLI: `mirket catalog read design-inspiration --path references/design-languages/duolingo-performance.md`.
- MCP: `skill_read` with `{"skill_id":"design-inspiration","path":"references/design-languages/duolingo-performance.md"}`.
- Executable source: `mirket://skills/design-inspiration/assets/design-languages/duolingo-performance/index.html`; retrieve using the same owner and `path: "assets/design-languages/duolingo-performance/index.html"`. Reading source is not playback evidence.
- Render the installed owner’s `assets/design-languages` with a local static server. Checkout command: `python3 -m http.server 8843 --bind 127.0.0.1 --directory skills/design-inspiration/assets/design-languages`. Visit `http://127.0.0.1:8843/duolingo-performance/` and `?variant=misleading`.

The complete question, static character and answer key are readable without JavaScript; inactive controls are hidden. Keyboard activation and visible focus remain available when enhanced. Current transforms are captured before canceling an old performance, and stale completion promises cannot own a newer event. Reduced motion at load or live change shows the chosen final pose without travel; page exit cancels finite animations. The example reflows at 320 px. Browser evidence establishes exercised paths, not human approval, native app behavior or measured learning gains.
