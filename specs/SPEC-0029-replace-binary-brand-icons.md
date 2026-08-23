---
id: SPEC-0029
title: Replace the inherited binary GUI brand icons with Orcker artwork
phase: 0
covers: [FR-001]
depends_on: [SPEC-0001, SPEC-0036]
surface:
  - apps/orcker-gui/
status: draft
attempts: 0
---

## Context

SPEC-0001 replaced the four Yerd `.svg` source marks with text placeholders but
left every rendered binary icon in place: `icon.png`, `icon.icns`, `icon.ico`,
`32x32.png`, `128x128@2x.png`, `StoreLogo.png`, the `Square*Logo.png` set and
the Android `mipmap-*` set under `apps/orcker-gui/src-tauri/icons/`, plus the
two DMG backgrounds. They carry no `yerd` string, so AC3 of SPEC-0001 passes,
but they are still Yerd's rendered "Y" mark and ship as Orcker's app icon.

## Requirements

- R1. Regenerate every binary icon under `apps/orcker-gui/src-tauri/icons/` from
  an Orcker source mark, so no Yerd artwork remains in the bundle.
- R2. Replace the two DMG background images, which carry the Yerd mark too.
