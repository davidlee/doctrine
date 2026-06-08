# Inquisition — SL-024 (design.md)

> **HERESIS URITOR; DOCTRINA MANET**

Target: `.doctrine/slice/024/design.md` (pre-plan). Doctrine consulted:
`slice-024.md` scope, ADR-001 layering, the storage rule, the
behaviour-preservation gate, and the body of the accused (`src/*.rs`,
`install/templates/*.toml`). The design was put to the question; most of its
confession holds true under iron. Two heresies and a venial taint remain.

## Charges

### CHARGE 1 — FALSE WITNESS on the spec test surface (medium)

**Doctrine violated:** correctness-first; "ask, don't infer"; an authored design
must not assert verification it did not perform (§10 A2 styles itself a
*confirmed* adversarial pass).

**The accusation.** § 9 and § 10 (A2) both swear that **every** module already
round-trips its `render_*_toml` output through `meta::Meta`, and name the
witnesses: *"adr.rs:232, slice.rs:559, requirement.rs:275, **spec.rs:1144**,
backlog.rs:1020"* — then prescribe *"One test fn per module, extending each
module's existing `render_*_toml_round_trips` test."*

**Confessed under cross-examination — false for spec:**

- `spec.rs:1144` is **not an assertion**. It is a section comment:
  `// --- VT-2: shared Meta round-trip + the member-count column ---`.
- The only spec round-trip, `spec_list_meta_parses_scaffolded_spec_toml`
  (`src/spec.rs:1145`), does **not** parse a `render_spec_toml` string. It calls
  `fresh(...)` to write the scaffold to **disk**, then `meta::read_meta(&tree, …)`
  reads it **back from disk**. The render output is never round-tripped directly.
- There is **no** `render_spec_toml_round_trips` test to extend. The four
  honest witnesses — `render_adr_toml_round_trips_to_metadata` (`adr.rs:229`),
  `render_toml_round_trips_to_metadata` (`slice.rs:557`),
  `render_requirement_toml_round_trips_to_metadata` (`requirement.rs:272`),
  `rendered_toml_round_trips_into_meta_and_backlog_item` (`backlog.rs:1015`) —
  each call the render fn and `toml::from_str(&body)`. Spec alone does not.

**Risk.** The plan that descends from this design will instruct the executor to
*extend* a spec test that does not exist, and to drive it with the same direct
`render_spec_toml(...) → toml::from_str` shape — which is in fact the **correct**
remedy, but the design denies it is new work. Worse, were the executor to instead
take the design's "via its reader" wording literally and route the hostile input
through the disk path (`fresh`), an explicit `--slug` bearing `"`/newline would
strike the **`<id>-<slug>` symlink creation** at the filesystem layer *before*
any TOML round-trip — a false-red from the wrong stratum (cf. the A1 false-red
the design already congratulates itself for catching).

**Sentencing.** Amend § 9 and § 10 A2: spec has **no** existing direct
render-round-trip test; SL-024 must **author a new** per-module test that calls
the private `render_spec_toml` directly and `toml::from_str`s the body (the
private fn is reachable — `mod tests` is in-file). Strike the false
"spec.rs:1144" witness and the "extending each module's existing test" universal;
it holds for four modules, not five. *Let the careless citation be read aloud at
the stake.*

### CHARGE 2 — STALE WITNESS in the scope (low)

**Doctrine violated:** authored-artifact accuracy (the storage rule's "reviewed,
diffable").

`slice-024.md` cites spec's splices at **`spec.rs:249` (slug) / `:250` (title)**.
The body confesses otherwise: `render_spec_toml` splices at **`spec.rs:260`
(slug) / `:261` (title)** — drift of eleven lines. The other four scope citations
(adr `74/75`, slice `71/72`, requirement `120/121`, backlog `452/453`) all still
hold true. Spec alone has wandered.

**Risk.** Low. Scope line-refs are advisory and the design does not repeat them.
But an authored artifact carrying a stale coordinate is a small rot that spreads.

**Sentencing.** Correct `:249/:250` → `:260/:261` in `slice-024.md`, or strike
the per-line precision in favour of the fn name. Minor penance: a day in the
stocks.

### CHARGE 3 — the disk-path wrinkle, absolved as written (observation)

The design's per-renderer test calls the render fn **directly** for the four
honest modules, rightly sidestepping symlink creation. This is correct and is
**not** charged — it is recorded only to bind Charge 1's remedy: spec's new test
**must** follow the same direct-render shape, never the `fresh`/disk path, lest a
hostile slug fail at the symlink stratum and counterfeit a red.

## Questions

1. **Q-1 (Charge 1).** Confirm the remedy: a **new** `render_spec_toml`
   direct-round-trip test (not an extension), driven by a `"`+newline+`\` title
   and explicit `--slug`. Agreed?
2. **Q-2 (Charge 2).** Correct the spec line-refs in scope, or de-precision them
   to fn name?
3. **Q-3 (orthogonal, already deferred — confirm only).** OQ-2 `state.rs:336`
   (`{{name}}` raw into the runtime sheet) stays out of scope this slice? The
   body confirms it is a raw splice; the design defers it. No objection raised —
   merely demanding the deferral be conscious, not forgotten.

## Pronounce Judgement

**The design is, in its bones, ORTHODOX.** The architecture is sound and well
evidenced: the new leaf `src/tomlfmt.rs` (no collision — confirmed absent; mods
declared in `main.rs`) sits at honest altitude under ADR-001; the verbatim move
of `toml_string`/`toml_array_inner` from `memory.rs:653/661` (bodies confirmed
byte-for-byte, visibility correctly raised to `pub(crate)`) discharges the
behaviour gate by construction; the self-quoting convention matches the lone
correct precedent (`memory.toml` bare token — confirmed); seven templates and
five renderers tally exactly with the tree; escape-only-`title`+`slug` (D5) is
correctly reasoned; the A1 self-correction (`]` is no string-literal breaker) is
itself true.

But the design **bears false witness about spec** (Charge 1): it swears a
uniform "extend the existing test" plan that does not exist for one of five
modules, and cites a comment line as an assertion. This is not a heresy that
sinks the design — it is a **taint to be cauterized before `/plan`**, lest the
plan inherit the lie and dispatch an executor to hunt a phantom test.

**Verdict: minor heresy. Remediable without re-architecture. Not fit for `/plan`
until Charge 1 is amended.**

## Sentencing (ordered)

1. **Amend § 9 + § 10 A2** — strike the `spec.rs:1144` witness and the "each
   module's existing `render_*_toml_round_trips`" universal; state plainly that
   spec gets a **new** direct-render round-trip test (four extended + one new).
   *Verification:* re-read § 9; no surviving claim that spec has an existing
   direct render-round-trip test. **Punishment for relapse: the wheel.**
2. **Correct `slice-024.md`** spec line-refs `:249/:250` → `:260/:261` (or
   de-precision). *Verification:* `grep -n 'replace("{{slug}}"' src/spec.rs` ⇒
   `260`. **Punishment: the stocks, one day.**
3. **Record the deferral of `state.rs:336`** as conscious (it already is, OQ-2) —
   no action beyond confirming Q-3.
4. **Then, and only then, proceed to `/plan`.** *Verification:* design re-read
   end-to-end; Charges 1–2 closed; the four honest round-trip tests + one new
   spec test enumerated as the red→green evidence.

> **HERESIS URITOR; DOCTRINA MANET**
