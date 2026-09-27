# ISS-497: Stale binary memory sync rolls back the shipped corpus

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`doctrine memory sync` overwrites `.doctrine/memory/shipped/` from whatever
binary runs it. The session-start hook runs it, so any harness still holding an
older build (a nix store path locked until the harness restarts) silently rolls
the shipped corpus back to its embed, and the next `doctrine boot` from that
session re-inlines the old content.

Observed 2026-09-27 (RFC-033): after `d947dd363` dropped the `onboarding` tag
from `mem.signpost.doctrine.overview`, the tag reappeared in the shipped copy
twice, each time after a stale-binary session started; boot re-inlined the
82-line overview until a fresh binary re-synced.

Candidate fix: sync refuses, or warns and skips, when the running embed is older
than the materialised corpus — e.g. a version or content stamp written at sync
and compared on the next run. STD-003: a skipped downgrade must be disclosed.
