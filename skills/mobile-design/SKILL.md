---
name: mobile-design
description: Design or review mobile journeys, navigation, touch targets, keyboard behavior, adaptive layouts and accessibility for iOS, Android or mobile web. Use for mobile product design and platform adaptation; native implementation still uses the project's actual platform workflow.
---

# Mobile design

Design the journey on the actual platform, including its interrupted and awkward
states. A narrow browser screenshot is useful evidence of reflow, not proof of a
native app or physical touch experience.

## Choose the platform and task

Identify iOS/iPadOS, Android, responsive web or a shared framework; supported
versions; actual devices; and the user's main task. Inspect existing navigation,
components and state ownership. Preserve platform conventions unless the product
has a justified exception. Use the available native engineering workflow for
implementation. `emil-mobile-native` is a mobile-web defect recipe, not an iOS or
Android SDK. Do not change frameworks or add a dependency to apply this skill.

Map the shortest meaningful journey: entry/deep link, selection/input, detail,
commit and return. Include pending, empty, failure and recovery where the real
contract needs them. Specify what survives rotation, window change, backgrounding
or process recreation; distinguish a local visual selection from saved data.

## Compose around input and available space

- Separate top-level destinations from drill-down and transient actions. Preserve
  destination state, platform back behavior and access to primary actions. A
  modal is not a substitute for every detail screen.
- Size interactive hit areas, not merely the drawn glyph. Use the platform's
  current guidance; iOS points, Android dp and CSS pixels are different units.
  Expanded hit regions cannot overlap adjacent actions. Do not make an essential
  action swipe-only, hover-only or dependent on precise dragging.
- Use text styles/scaling from the actual platform. Exercise long translations,
  large accessibility text, RTL where supported and keyboard focus order. Truncate
  only when users can still obtain the necessary value.
- Plan for keyboard and system insets: focused field and submit/recovery action
  remain reachable, scrolling has one clear owner, and dismissal preserves input.
  Never hard-code a keyboard height or treat every phone as a fixed rectangle.
- Select platform controls for text entry, pickers, disclosure and destructive
  actions where appropriate. Set input purpose, labels, validation and autofill
  from the real data contract; visual resemblance does not provide semantics.
- Give selection and errors a readable non-color cue. Check screen-reader names,
  role, state, grouping, focus movement and actionable order. Decorative artwork
  should not become a series of noisy accessibility nodes.

Read [the worked mobile journey](references/materials-journey.md) for a concrete
platform comparison and acceptance probes. Use its guidance selectively; it does
not prescribe bottom navigation or large targets to unrelated desktop surfaces.

## Verify the right claim

Use the host's browser tools or explicitly registered local tools for mobile web:
compact/wide layouts, normal/reduced motion, input/focus and failure recovery.
Use native simulator/emulator tests for the built app when available, then physical
hardware for important touch, keyboard, safe-area, gesture and performance claims.
Do not present a browser user-agent string or screenshot as a native build.

Exercise real Back/Escape/dismissal, rapid repeats, keyboard show/hide, interrupted
submission, relevant offline state and restoration. Confirm the durable effect
through reload or a second read when persistence is part of the task. Test
VoiceOver/TalkBack with the actual screen rather than inferring accessibility from
labels alone. Record the OS, browser/build, device and input path actually used.

Deliver the working flow, relevant states, inspected captures/playback and concrete
remaining defects. Use `interface-design` for composition, `motion-design` for
ordinary web motion and the real native animation APIs for a native app. Separate
tested browser reflow, native build checks, device checks and human acceptance.
