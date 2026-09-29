# IDE-062: jgrep known-answer probe

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Cheapest real test of Jev relevance before any SL-277 build. `jgrep` (npm,
runs outside the jail) does Jev semantic search over a directory.

First look (2026-09-29, `jgrep "related to CLI UX" .doctrine/requirement/`):
32 hits over 2622 chunks, about 410k tokens, $0.017, 3.1s. Strong hits were
genuinely CLI-UX requirements that `doctrine search` misses, since they never
use the word "CLI". Weaknesses: TOML chunks such as `tags = []` score about 0.75,
so scores near 0.7 are noise; one entity appears 2-4 times in fixed windows.

Probe: take a few closed slices' `research.md` queries, run jgrep over
`.doctrine/` Markdown only, and check whether the cited entities come back,
against `doctrine search` on the same query. Lo-fi, scripted, off-lifecycle.
If it wins, SL-277 (benched) or a leaner slice follows.

Origin: IDE-061 (the Jev relevance trial idea, promoted to SL-277).
