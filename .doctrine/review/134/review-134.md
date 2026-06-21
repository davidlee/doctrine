# Review RV-134 — design of SL-139

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Lines of interrogation for SL-139's design:

1. **Show-parity scope.** The design claims to "normalize show parity" but only identifies one concrete deviation: concept-map's missing `--json` shorthand. Does the design define what the normalized end state actually is, or does it conflate CLI-flag existence with JSON-output-shape uniformity?
2. **Layering.** The design proposes `src/paths.rs` but defers its ADR-001 tier assignment to implementation. A module that scans the filesystem but imports no clap — where does it belong?
3. **Coverage.** Does the 13-command enumeration correctly account for sub-kind dispatch (backlog's 5 kinds, spec's 2 kinds, knowledge's 4 kinds)?
4. **Completeness.** Are there edge cases the design omits? MCP surface implications, hidden files in entity directories, concept-map's pre-existing body tolerance?

## Synthesis

**Judgement: The design is NOT heresy — it is incomplete in five specific points, all minor-to-moderate, all fixable at the design artifact.** The core architecture (shared paths helper, per-kind adapters, `--single`/`--toml`/`--md`/`--entity` selectors, splat-atomic output) is sound. The deviations from doctrine are omissions, not contradictions.

### Ordered Penance

1. **Add a decision (D-n) clarifying show-parity scope** (F-1, major/design-wrong). Insert: parity means CLI-grammar parity (every kind accepts `--json` shorthand). JSON-output-shape normalization is deferred to IMP-145 and is explicitly out of scope for SL-139.
2. **Assign `src/paths.rs` an explicit ADR-001 tier** (F-2, major/fixed). Insert: engine tier. Rationale: entity.rs already does filesystem I/O in the engine tier; path projection is adjacent. Command depends on engine (downward, correct).
3. **Specify sub-kind directory paths for umbrella commands** (F-4, minor/fixed). Insert: backlog → `.doctrine/backlog/{issue|improvement|chore|risk|idea}/NNN/`; spec → `.doctrine/spec/{product|tech}/NNN/`; knowledge → `.doctrine/knowledge/{assumption|decision|question|constraint}/NNN/`.
4. **Define the exclusion filter for non-authored regular files** (F-7, minor/fixed). Insert: exclude dot-prefixed, editor-temporary, and known-artifact patterns from `paths` output.
5. **Acknowledge concept-map's pre-existing body tolerance** (F-6, minor/fixed). Insert: note that `concept_map::read_concept_map` tolerates missing `.md` via `unwrap_or_default()`; `show` preserves this tolerance; `paths --md` enforces strictness.
6. **Explicitly exclude MCP from scope** (F-3, minor/tolerated). Insert: "Do not add an MCP `paths` surface" to Non-Goals. MCP tools reflect the CLI; a future binding is follow-up.
7. **`--entity` flag is syntactic sugar** (F-5, nit/tolerated). Acknowledged; the discoverability benefit justifies the trivial cost.

### Standing Risks

- **Per-kind adapter duplication** (design §8): the design names the risk but doesn't specify the trigger threshold. If more than 5 of the 13 kinds need custom path-construction logic beyond "identity TOML + MD in a flat directory," revisit before merging.
- **`paths` verb expands SPEC-013 verb set** (design D7): the spec-drift reconciliation is scheduled for the reconcile phase. Must not be forgotten.

### Tolerated Taints

- MCP surface omission (F-3): downstream, low urgency, no correctness impact.
- `--entity` sugar flag (F-5): trivial cost, acceptable discoverability win.

**The Inquisition declares this design purified by confession. Let the corrections be inscribed before the first phase sheet is cut, lest the implementer wander into the wilderness of ambiguity.**

> **HERESIS URITOR; DOCTRINA MANET**
