# Review RV-173 — design of SL-164

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Inquisition of the **design intent** of SL-164 (`.doctrine/slice/164/design.md`) —
wiring memory write verbs + onboarding into the MCP server. The accused promises
"all design decisions resolved" (§ Remaining open questions). The Inquisitor
presumes that confidence is heresy until proven.

**Lines of interrogation:**

1. **Phantom API** — does every function the dispatch code invokes actually exist
   in the codebase? (`parse_optional_status`, `parse_optional_lifespan`,
   `MemoryType::parse`.) Does the design declare new engine code where it invokes
   non-existent helpers?
2. **Type fidelity** — does the prescribed `RecordArgs` construction match the
   real struct field types (`status: Status` non-optional, `lifespan:
   Option<Lifespan>`)?
3. **Error-mapping truth** — does `.context("invalid arguments")` actually satisfy
   `map_review_error`'s `msg.starts_with("invalid arguments:")` gate, given anyhow
   `to_string()` shows only the top context? Do the design's own VT assertions
   (-32602) hold?
4. **Blast radius honesty** — does adding `writer` to `run_record`/`run_edit`
   break callers the design's impact table conceals? (AGENTS.md
   behaviour-preservation gate: existing suites must stay green.)
5. **Pattern fidelity / DRY** (CLAUDE.md) — `dyn Write` vs the cited `impl Write`
   pattern; reuse of existing parsers vs parallel implementation.
6. **Stale relics** — propagated misnamed tests / doc headers (`tool_list_has_14_tools`,
   the `14 tools / 4 memory` module doc).

**Doctrine held against the accused:** ADR-001 (module layering), AGENTS.md
behaviour-preservation gate + no-parallel-implementation, CLAUDE.md DRY / "write
less code", and the design's own internal coherence (§ Verification impact must
agree with § Dispatch).

## Synthesis — the verdict

**HERESY FOUND. The design does not lock.** Its closing boast — "Remaining open
questions: None — all design decisions resolved" — is itself the first heresy:
the artifact carries three blockers that would refuse to compile or fail its own
verification tests. The accused confessed under cross-examination of the source.

**Blockers (gate the design's advance):**
- **F-1** — the dispatch invokes `parse_optional_status` / `parse_optional_lifespan`,
  functions that exist NOWHERE in the tree, while the design swears "no new engine
  code." Phantom API. Reuse the sanctioned `Status::parse` / `Lifespan` parser.
- **F-2** — `RecordArgs.status` is a non-optional `Status` (default `Active`); the
  design builds it as an Option. Type mismatch — will not compile.
- **F-3** — `.context("invalid arguments")` (no colon) cannot satisfy
  `map_review_error`'s `starts_with("invalid arguments:")`; anyhow's `to_string()`
  shows only the top context. `memory_edit`-no-flags and empty-title errors map to
  `-32603`, not `-32602` — the design's own VTs would FAIL.

**Major:**
- **F-4** — adding `writer` to `run_record` breaks ~35 callers (boot.rs ×4,
  retrieve.rs, ~28 memory.rs tests). The impact table names ONE and omits
  `src/boot.rs` / `src/retrieve.rs` entirely — a breach of the behaviour-preservation
  gate by omission.

**Minor / nit:** F-5 (`dyn Write` contradicts the cited `impl Write` pattern),
F-6 (the count bump propagates the lying `tool_list_has_14_tools` name and ignores
the stale `tools.rs:4` module-doc header), F-7 (stderr notice silently dropped —
tolerated).

**Ordered penance** (apply in the design before it locks; re-enter `/design`):
1. F-1/F-2 — rewrite the `memory_record` dispatch to parse via existing enum
   parsers; resolve `status` to a concrete `Status` (default `Active`), `lifespan`
   to `Option<Lifespan>`.
2. F-3 — colon-suffix the `invalid arguments:` wrapping on the run_record/run_edit
   error paths; correct the false "hits branch 2" claim. Verify by the existing
   `-32602` VTs once code lands.
3. F-4 — enumerate every `run_record` caller; expand the impact table to include
   `src/boot.rs` + `src/retrieve.rs`; state the test-caller `&mut io::stdout()` fix.
4. F-5 — `&mut impl Write`. F-6 — rename the test, fix the module-doc header.

**Standing risks consciously left:** F-7 tolerated (advisory stderr). Also noted,
not raised: `retrieve_reference` runs `check_retrievable`/staleness on the
single-memory path — if a signpost is ever held back or stale, `doctrine_onboard`
would silently emit less than promised. Worth a sentence in the design's
onboarding section.

**Sentence:** the blockers remain UNVERIFIED and GATING. The design is refused
until the penance is wrought and the charges verified. Until then — the stake
stands ready.

> **HERESIS URITOR; DOCTRINA MANET**
