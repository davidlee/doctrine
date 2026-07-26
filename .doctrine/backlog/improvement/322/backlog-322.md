# IMP-322: Make Pi research runners tolerate read-only session homes

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

During SL-231 planning, both project-mandated research producers
(`scripts/pi-research` and `scripts/pi-scout`) failed before repository
inspection. Pi attempted to create settings locks and session directories under
read-only `/home/david/.pi`, returning EROFS. The plan used the documented
orchestrator-run fallback, but lost the independent research arms.

## Desired outcome

Give the research runners an explicit writable, disposable Pi settings/session
home under an approved workspace, cache, or temporary root. Preserve their
read-only repository posture and ensure cleanup or bounded retention. Verify
both runners can start and write their raw research artefacts when the invoking
agent's normal home is read-only.
