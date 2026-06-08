# Design SL-027: DRY backlog test-fixture TOML builders into one helper

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, ISS-001, ADR-004); doc-local refs bare — D1, R1. -->

## 1. Design Problem

The `#[cfg(test)] mod tests` block of `src/backlog.rs` holds three helpers that
each hand-build a `backlog-NNN.toml` fixture and write it to a temp tree:

- `write_item` (`src/backlog.rs:1417`) — base item: core head + `tags`.
- `write_assessed_risk` (`src/backlog.rs:1928`) — risk: core head + `[facet]` +
  empty `[relationships]`.
- `write_related` (`src/backlog.rs:1943`) — item + seeded `[relationships]`.

All three duplicate the same TOML literal — the core field head
(`id`/`slug`/`title`/`kind`/`status`/`resolution`/`created`/`updated`/`tags`),
the `2026-06-08` date literals, the `backlog-{name}.toml` path computation, and a
`"quote"`-and-join list-literal closure (`write_item`'s `tags_lit`,
`write_related`'s `lit`). They diverge only in the optional `[facet]` and
`[relationships]` trailers.

A **fourth** copy of the same head literal hides inline at `src/backlog.rs:1813`
(`backlog_show_json_is_faithful_item_state`): a hand-written *fully-assessed
risk* (facet + populated relationships + custom status/resolution/tags) written
to disk and read back. It was inlined because the old narrow helpers could not
express "facet AND populated relationships AND custom head" at once — the unified
builder can, so it folds in (see §3.4).

Captured as **ISS-001** (`backlog-test-fixture-dry`), born of SL-020. A schema
field added or renamed means editing the literal in **four** places; the quoting
closure is copy-pasted twice. This is **test-only** debt: no production surface
and no behaviour changes.

### 1.1 Scope boundary — inline literals that stay

Two other inline TOML literals in the test module are **not** head-copies and
stay explicit by design:

- `:1161` (`risk_facet_levels…parse`) — an **in-memory** string fed straight to
  `toml::from_str`; never written to disk. A parser round-trip fixture where the
  exact bytes under test must be visible. `write_fixture` writes to disk — wrong
  shape.
- `:2004` (`backlog_edit_is_edit_preserving`) — builds on the **real producer**
  (`new_item`) then hand-appends an inert `[custom]` table; not a head-literal
  copy.

## 2. Locked Decision (D1) — extract a shared core, keep thin named wrappers

The duplication ISS-001 names lives in the three **function bodies**, not the
call sites: `write_item` already centralises its 25 calls behind one signature.
So the highest-leverage, lowest-risk move is to lift the shared literal into one
core builder and let the three helpers delegate as thin (≤4-line) named wrappers.

Rejected alternative: collapse to a single fluent/struct builder and delete the
named helpers. That rewrites ~30 call sites (most regression surface, for a
cosmetic "one entry point" win) and loses the self-documenting helper names
(`write_assessed_risk` announces intent at the call site). It wins only a
literal-vs-spirit reading of "one helper" at a net cost under DRY and the
behaviour-preservation gate.

"One helper" is satisfied in substance: **one** core builder owns the literal;
the wrappers are adapters, not duplicated builders.

## 3. Target Design

### 3.1 Types (test module; lifetime-parameterised)

`write_related` passes a **borrowed** `slices: &[&str]`, so the spec carries a
lifetime `'a` rather than `'static`. Call sites passing `&'static` literals
satisfy any `'a`.

```rust
struct Fixture<'a> {
    kind: ItemKind,
    id: u32,
    slug: &'a str,
    title: &'a str,
    status: &'a str,
    resolution: &'a str,
    tags: &'a [&'a str],
    facet: Option<FacetLit<'a>>,   // None → no [facet] block
    rels:  Option<RelLit<'a>>,     // None → no [relationships] block
}
struct FacetLit<'a> { likelihood: &'a str, impact: &'a str, origin: &'a str, controls: &'a [&'a str] }
struct RelLit<'a>   { slices: &'a [&'a str], specs: &'a [&'a str] }
```

### 3.2 Core (the three single-source seams)

```rust
// sole list-literal quoting (replaces tags_lit + lit closures)
fn toml_list(xs: &[&str]) -> String;            // [] → "" ; ["a","b"] → "\"a\", \"b\""

// sole TOML literal: core head + optional [facet] + optional [relationships]
fn render_fixture_toml(f: &Fixture) -> String;

// sole path/dir/write
fn write_fixture(root: &Path, f: Fixture);
```

`render_fixture_toml` assembles `head + facet_segment + rels_segment` by
`format!` concatenation (each optional segment is `""` when absent) — **not**
`push_str(&format!(..))`, honouring the repo string-build convention
(`mem.pattern.lint.string-build-no-push-format`) even though `cargo clippy` does
not lint `cfg(test)` code.

### 3.3 Wrappers (named intent retained; all ~30 call sites untouched)

```rust
fn write_item(root, kind, id, status, resolution, slug, title, tags)   // facet:None, rels:None
fn write_assessed_risk(root, id)                                       // facet:Some(..), rels:Some(empty)
fn write_related(root, kind, id, slices, specs)                        // facet:None, rels:Some(slices,specs)
```

Signatures are unchanged, so the 25 + 2 + 3 existing call sites compile and run
verbatim.

### 3.4 Folding the inline 4th copy (`:1813`)

The only **call-site change** in the slice: the hand-written literal in
`backlog_show_json_is_faithful_item_state` becomes one `write_fixture` call. The
unified builder reproduces it byte-for-byte and at the same path:

```rust
write_fixture(root, Fixture {
    kind: ItemKind::Risk, id: 1, slug: "leak", title: "Token leak",
    status: "resolved", resolution: "mitigated", tags: &["security"],
    facet: Some(FacetLit { likelihood: "high", impact: "critical", origin: "audit", controls: &["rotate"] }),
    rels:  Some(RelLit { slices: &["SL-020"], specs: &[] }),
});
```

The test's own `fs::create_dir_all(dir2)` + `fs::write` lines are deleted —
`write_fixture` does both. The `read_item` + `show_json` assertions are untouched
and remain the proof.

## 4. Behaviour Preservation — byte-identical output

The gate (CLAUDE.md): changing shared machinery, the existing suites are the
proof — they must stay green **unchanged**. That holds iff the unified builder
emits byte-identical TOML. Verified field order + blank-line spacing against all
three current bodies:

- head ends `tags = [..]\n`; `[facet]` segment leads with `\n[facet]` →
  `tags = []\n\n[facet]` ✓ (blank line preserved)
- `[facet]` ends `controls = [..]\n`; `[relationships]` segment leads `\n[relationships]` ✓
- `[relationships]` ends `drift = []\n` ✓
- `write_item` (no trailers) = head only, no trailing blank line ✓
- `:1813` fully-assessed risk — head (`tags = ["security"]`) + facet + rels;
  reproduced exactly, same `risk/001` path ✓
- `toml_list`: `&[] → ""`, `&["a"] → "\"a\""`, `&["a","b"] → "\"a\", \"b\""` —
  matches both old closures ✓

Same bytes in → identical parse/assert downstream → suite green.

## 5. Verification

- **No new tests.** This is a refactor; the existing backlog suite staying green
  **unchanged** is the behaviour-preservation gate and the whole proof.
- `just check` (fmt + clippy zero-warnings + test + build) green.
- **Closure check:** `grep -c 'created = \"2026-06-08\"'` over `src/backlog.rs`
  drops from **7 → 4**. The four *fixture-builder* copies (`write_item` :1439,
  `:1813` :1809, `write_assessed_risk` :1934, `write_related` :1955) collapse to
  **one** (the unified builder). The three survivors are deliberately-explicit
  **in-memory / error-path** inputs that must show their exact bytes, not fixture
  builders: `:1157` (`:1161` parser round-trip), `:1190`
  (`validate_errors_on_an_unknown_enum_token`), `:2075`
  (`backlog_edit_refuses_malformed…`).
- ISS-001 transitioned to its resolving state at `/close`.

## 6. Scope / Phasing

Single phase — too small to split. Mechanical extract-and-delegate with a
byte-equivalence invariant; no sequencing risk.

## 7. Risks (carried from slice-027.md)

- **R1 over-parameterisation** — *retired*. The wrapper approach keeps existing
  signatures; no mega-signature is introduced. (The clippy arg-ceiling sub-fear
  was already moot: the gate's plain `cargo clippy` does not compile/lint
  `cfg(test)`.)
- **R2 call-site readability** — *retired by D1*. Named wrappers preserve intent;
  call sites are byte-for-byte unchanged.
- **Residual risk** — a transcription error in the unified literal breaks output
  equivalence. Mitigated by §4's explicit byte-level check and the unchanged
  downstream assertions catching any drift immediately.
