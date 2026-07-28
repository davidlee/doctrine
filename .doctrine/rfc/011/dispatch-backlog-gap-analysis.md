# RFC-011 Dispatch Backlog Gap Analysis

> Generated 2026-07-28 from full corpus of case notes across 6 files.
> Maps every dispatch-related friction entry to existing or newly-created backlog items.

## Summary

| Metric | Count |
|---|---|
| Total case-note entries across all files | ~290 |
| Dispatch-workflow entries | ~150 |
| Already covered by existing backlog | ~40 items |
| **Newly created backlog items (this pass)** | **13** |
| Low-severity / convention-level entries not backlogged | ~5 |

## New Backlog Items Created

| ID | Title | Source(s) |
|---|---|---|
| IMP-333 | dispatch_import undeclared-scope refusal returns empty detail | SL-210, SL-213, SL-222 |
| IMP-334 | Claude arm isolation:worktree omission footgun — add pre-spawn check | SL-199-armswitch-a7, SL-206-P01-forkprobe |
| IMP-335 | Entity creation prints only canonical dir, silently creating tracked slug symlink | SL-205-reconcile, SL-212-REV-030 |
| IMP-336 | review dispose --as should accept participant display labels as aliases | RV-252, SL-225-RV291, SL-203 |
| IMP-337 | Worker absolute-path reads silently hit primary tree instead of fork | SL-220-P06, SL-221-P02, IMP-261 |
| IMP-338 | Session scratchpad / runtime state wiped mid-drive loses per-phase evidence | SL-231-conclude |
| IMP-339 | Workflow driver scripts must conform to actual Claude Code harness contract | SL-206-drive-p05-harness-contract |
| IMP-340 | Worker negative contract: never delete tests without explicit authorization | SL-222-P09 |
| IMP-341 | RustEmbed embed-root additions don't trigger cargo rebuild | SL-206-drive-p05-bootstrap, SL-193-audit |
| IMP-342 | Subagent worktree-jail hook blocks read-only CLI reads from delegated subagents | RFC-016-cluster2-preflight |
| IMP-343 | backlog body filename pattern inconsistent across entity kinds | sess-graph-cli |
| IMP-344 | Design line-number citations rot fast — cite symbol names instead | SL-212-pp-05, SL-232-design-write-a |
| IMP-345 | boot snapshot SPINE lists 'reports'/'explore' as group headings not subcommands | SL-224-audit, SL-233-RV315 |

## Dispatch Entries Already Covered by Existing Backlog

| Case-Note Pattern | Existing Backlog |
|---|---|
| prepare-review phase-status split-brain (coord vs primary) | IMP-272, IDE-028 |
| Object-db imports leave coord worktree stale (reverse-diff) | IMP-300, ISS-234 |
| worker_commit gate false-red (stale PATH binary) | ISS-218, IMP-270 |
| selector under-declaration (design-target vs scope-relevant) | IMP-256, ISS-251 |
| Split-lineage close friction (dispatched slice 3-way merge) | IMP-127, IMP-236, IMP-174, IMP-201 |
| e2e authored-write goldens fail under worker marker | CHR-044, ISS-028 |
| dispatch-agent SKILL documents retired funnel | IMP-271 |
| verify-vt UNATTRIBUTABLE false-negative in coord topology | ISS-226, IMP-228 |
| arm-spawn --phase omission half-arm trap | IMP-331 |
| worker_commit red-gate embeds full transcript (~295k chars) | ISS-219 |
| candidate status/admit flag/hint mismatch | IMP-233 |
| architecture_layering blocks worker on legitimate changes | IMP-293 |
| stale PATH binary for census/corpus inspection | ISS-053 |
| check regression base sha normalization | IMP-278 |
| boot --check exits 0 even when stale | ISS-228 |
| check gate eslint leg fails in jail | ISS-222 |
| phase scope gaps: plan touch-sets miss transitive fallout | IMP-256 |
| stale coord tree blocks verify-vt at conclude | ISS-252 |
| claude-arm worker_commit false-red on non-hermetic ambient tests | ISS-235 |
| Conflicting-merge dispatched slice uncloseable by machine | IMP-127, IMP-236 |
| dispatch_reap not-landed refusal prescribes itself | ISS-250 |
| dispatch_arm writes to wrong root when CWD != coord | IMP-233, IMP-268 |
| conformance --strict flags import-landed funnel row | ISS-264 |
| boundary row spanning mid-drive refresh-base | ISS-268 |
| slice conformance reads boundaries from primary, phases from cwd | ISS-269 |
| conclude cadence re-arms unfollowable refresh-base post teardown | ISS-270 |
| verify-vt FAIL for absent test_file, not UNCHECKABLE | ISS-271 |

## Low-Severity / Convention Items (Not Backlogged)

These are genuine friction but individually low-impact, better addressed as convention updates:

- `echo ===` breaks under zsh — document in AGENTS.md (use `echo '==='` or `printf`)
- `doctrine memory retrieve` rejects positional uid — CLI UX papercut
- Section-addressable read for long `show` output — nice-to-have
- Nested spawn topology: probe cheaply before full drive — skill guidance, not a code fix
- `cargo test` green ≠ `clippy` green (handover miscertifies) — skill guidance
- plan.toml scaffold multi-line inline table example — covered by IMP-176 (validation) but scaffold fix is separate

## Methodology

1. Read all 6 case-note files (~8,500 lines total): `case-notes.md`, `2026-07-02.case-notes.md`, `2026-07-04.case-notes.md`, `2026-07-20.case-notes.md`, `2026-07-27.case-notes.md`, `case-notes-analysis.md`
2. Extracted ~150 dispatch-workflow entries (entries tagged `[dispatch; ...]` or describing dispatch mechanics)
3. Cross-referenced against the existing `doctrine backlog list --tag area:dispatch` (80+ items)
4. For each entry without an existing backlog item: created a new `IMP-NNN` with appropriate area/cluster tags
5. Grouped remaining entries under their existing backlog items
