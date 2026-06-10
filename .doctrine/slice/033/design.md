# SL-033 Design — Standard (STD) governance kind

## 1. Decision summary

STD (`STD-NNN`) is the third governance kind, riding SL-030's `governance.rs`
spine **unchanged** — a thin data module exactly like `policy.rs`, plus the
boot-projection parameterization SL-030 deferred. No new spine mechanism.

**Locked decisions:**

- **D1 — STD is a sibling of POL, not its soft variant.** Both record *standing
  rules*; STD differs by gaining a `default` tier ("recommended unless justified
  to deviate", from the supekku `standard-template.md` prior art) distinct from
  `required` (mandatory). Topic, not enforcement level, separates STD from POL.
- **D2 — Status vocab:** `draft / default / required / deprecated / retired`.
  Template seeds `draft`. `default` + `required` are in-force; `deprecated`
  (sunsetting-extant) + `retired` (terminal-off) are the hide-set, mirroring POL.
- **D3 — Boot in-force is a SET, not a single literal.** STD has two in-force
  statuses, so the boot projection filters on `{default, required}`. Since
  `ListArgs.status` is already a `Vec<String>`, carrying a status set costs
  nothing and generalizes ADR (`{accepted}`) / POL (`{required}`) uniformly.
- **D4 — Supersession is a relationship, not a status** (inherited from POL D2):
  no `Superseded` variant; `relationships.supersedes` carries it.

## 2. Current vs target behaviour

| | Current | Target |
|---|---|---|
| `doctrine standard …` | absent | `new/list/show/status`, POL-identical |
| `.doctrine/standard/` | absent | authored tree (manifest dir + gitignore negation) |
| `standard.{toml,md}` templates | absent | rust-embedded, mirror `policy.*` |
| boot per-kind projection | one `SourceKind` variant + one `produce` arm **per kind** | one data-carrying variant + one arm for **all** governance kinds |
| boot "Active Standards" section | absent | after "Active Policies", before "Memory" |

## 3. Code impact

### 3.1 `src/standard.rs` (new — mirror `src/policy.rs`)

```rust
const STANDARD_DIR: &str = ".doctrine/standard";

pub(crate) const STD_KIND: GovKind = GovKind {
    kind: Kind { dir: STANDARD_DIR, prefix: "STD", scaffold: standard_scaffold },
    stem: "standard",
    statuses: STANDARD_STATUSES,
    hidden: is_hidden,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub(crate) enum StandardStatus { Draft, Default, Required, Deprecated, Retired }
// as_str: Draft→"draft", Default→"default", Required→"required",
//         Deprecated→"deprecated", Retired→"retired"

const STANDARD_STATUSES: &[&str] =
    &["draft", "default", "required", "deprecated", "retired"];

fn is_hidden(s: &str) -> bool { matches!(s, "deprecated" | "retired") }

// render_standard_toml / render_standard_md / standard_scaffold /
// run_new / run_list / run_show / run_status — structurally verbatim policy.rs,
// "policy"→"standard", "POL"→"STD".
```

`stem == prefix.to_lowercase()` here (`"standard"` / `"STD"`) — the POL case
already proved `stem != prefix` works, so this trivial case is safe.

### 3.2 Templates (rust-embedded under `install/templates/`)

- `standard.toml` — `schema = "doctrine.standard"`, body identical to
  `policy.toml` (id/slug/title/`status="draft"`/dates + inert `[relationships]`
  with `supersedes/superseded_by/related/tags`).
- `standard.md` — sections from `supekku/templates/standard-template.md`
  (Statement / Rationale / Scope / Verification / References). Attribute the
  prior art in a leading comment; **drop the YAML frontmatter** (D1 storage rule —
  metadata in the sister TOML). Carries the standard-specific note that a
  `default`-status standard is recommended-unless-justified.

### 3.3 CLI wiring (`src/main.rs`) — mirror POL

`mod standard;`; `Command::Standard { command: StandardCommand }`; the four
subcommands (New/List/Show/Status, `status: standard::StandardStatus`);
classifier arms (New/Status → Write, List/Show → Read); dispatch arms forwarding
to `standard::run_*`; a `standard_split` classifier test mirroring `policy_split`.

### 3.4 Install wiring

- `install/manifest.toml` — add `".doctrine/standard"` to the create-dirs.
- `.gitignore` — add `!.doctrine/standard/` negation (authored-entity-wiring).
- `install.rs` test — mirror `embedded_manifest_creates_the_policy_tree`.

### 3.5 Boot refactor (`src/boot.rs`) — the only non-mechanical change

Collapse the per-kind variants/arms into one data-carrying variant. **The new
variant is NOT named `Governance`** — that identifier already binds the
`governance.md` disk reader. Use `GovRows`:

```rust
// SourceKind: drop `Adrs` and `Policies`, add:
GovRows(&'static GovKind, &'static [&'static str]),   // kind + in-force status set

// boot_sequence():
("Accepted ADRs",    SourceKind::GovRows(&adr::ADR_KIND,       &["accepted"])),
("Active Policies",  SourceKind::GovRows(&policy::POLICY_KIND, &["required"])),
("Active Standards", SourceKind::GovRows(&standard::STD_KIND,  &["default", "required"])),

// produce():
SourceKind::GovRows(g, in_force) => section_or_marker(
    heading,
    governance::list_rows(
        g,
        root,
        crate::listing::ListArgs {
            status: in_force.iter().map(|s| (*s).to_string()).collect(),
            ..Default::default()
        },
    ),
),
```

**Byte-identity — by construction, not by test.** Output identity holds
*structurally*: headings remain string literals in `boot_sequence`, and ADR's
`["accepted"]` / POL's `["required"]` build the exact `ListArgs` the old arms
built (`status` Vec single-element, all other fields `..Default::default()` — the
old arms set nothing else), so `list_rows` is called with identical inputs and
emits identical bytes. The existing boot suites
(`regenerate_projects_accepted_adrs_and_memory_pointers`,
`regenerate_projects_required_policies_filtered`) are the *behaviour-preservation*
gate — they assert row presence, the exact `PREFIX-NNN  status` row format,
filtering (draft/deprecated/retired excluded), and section ordering, **run
unchanged**. They prove semantic preservation, not full-section byte equality; the
byte-identity itself rests on the construction argument above. If stronger
assurance is wanted, a byte-exact section golden is a cheap add (the ADR golden
pattern, `tests/e2e_adr_cli_golden.rs`) — not required, since the inputs are
provably unchanged.

### 3.6 Glossary

STD row already present (`STD-123`). No change (parity with SL-030 OQ-1).

## 4. Verification alignment

**New (standard.rs, mirror POL):** toml round-trip; hostile-escape; relationships
inert; md-no-frontmatter (`# STD-NNN:`, no `---`); scaffold 2-files+symlink;
`standard_known_set_matches_variants` (drift canary, 5 variants); hide-set ⊆
known-set; symbol-only-title-needs-`--slug` bail.

**main.rs:** `standard_split` classifier test.

**install.rs:** standard tree created + not gitignored (unit — surface 1 only).

**Conformance parity (the load-bearing gates POL/ADR already hold — mirror, don't
skip):**
- **install→commit e2e** (mirror `tests/e2e_policy_install_commit.rs`): the
  `!.doctrine/standard/` negation makes a scaffolded standard committable; without
  it, ignored. Catches the "scaffolded but uncommittable under the blanket
  `.doctrine/*` ignore" trap the install.rs unit test does NOT cover.
- **worker-guard matrix** (`tests/e2e_worker_guard.rs`): add `standard new` +
  `standard status` rows — both hard-refuse under `DOCTRINE_WORKER=1` with the
  named-verb error (ADR-006 INV; read verbs unaffected).
- **list parse-conformance** (`tests/e2e_list_conformance.rs`): extend to the
  governance kinds so `standard list` proves `--filter/--regexp/--status/--json`
  + the JSON envelope ride the shared spine (the matrix currently omits even POL).
- **black-box CLI golden** (mirror `tests/e2e_adr_cli_golden.rs`): byte-exact
  `standard list` (populated tree — hide-set + ordering + prefix + header),
  `standard show` (Table + `--json`), `standard status` (transition + no-op +
  malformed-refuse). A one-char render edit turns a golden red.

**boot.rs — gate (stay GREEN UNCHANGED):** existing ADR/POL ordering + filtered-
projection tests prove byte-identity. **New tests:**
- `boot_sequence_orders_active_standards_after_active_policies` (ordering).
- `regenerate_projects_in_force_standards_filtered` — STD section includes
  `default` + `required` rows, excludes `draft`/`deprecated`/`retired`. This is
  the **only** test proving the two-status in-force *set* (vs POL's single).

**Black-box goldens:** `standard --help`/`list` parity (black-box-cli-golden
pattern). Routing digest presence-checked only — new boot rows are golden-safe.

## 5. Verification taxonomy

All criteria above are `VT` (by test). No `VA`/`VH` introduced.

## 6. Risks, invariants, gotchas

- **R1 — rust-embed re-embed footgun:** a lone template edit is invisible until
  the embedding crate recompiles. The render tests read the embed, so they pin
  it; if a manual edit seems not to land, force a recompile.
- **R2 — jail build target:** `./target/debug/doctrine` is stale in the jail; the
  live binary is under `~/.cargo/doctrine-target-jail/debug`. Use the build target
  for manual CLI checks.
- **R3 — `Default` variant:** `default` is a contextual keyword only; the
  `Default` enum variant and clap-derived `"default"` string are safe.
- **Lint:** adding the `Standard` subcommand may approach the CLI-handler arg/bool
  ceiling (mem.pattern.lint.cli-handler-args-struct) — POL didn't trip it, parity
  expected. Gate: `cargo clippy` (bins/lib only, no `--all-targets`) zero warnings;
  `just check` before commit.
- **Invariants:** hide-set ⊆ known-set; boot in-force ⊆ known-set and disjoint
  from hide-set (`{default,required}` are visible + in-force, never hidden).

## 7. Non-goals (from scope)

No new spine mechanism; inherited shared gaps (boot error≡empty marker,
supersession⇏status, inert `--tag`) stay deferred; no `standard supersede` verb;
not an ADR, not a `doc/*` spec.

## 8. Internal adversarial pass

- **"Is the boot variant name collision real?"** Yes — `SourceKind::Governance`
  exists (the `governance.md` disk reader). The scope text proposed
  `SourceKind::Governance(...)`, which would collide. Resolved: `GovRows`.
- **"Does a status *set* break byte-identity for ADR/POL?"** No — single-element
  sets `["accepted"]`/`["required"]` build the same `ListArgs.status` Vec the old
  literals did. Identical bytes.
- **"Why not Option C (default-only, no `required`)?"** It would forbid mandatory
  standards — stricter than the supekku prior art, which allows a standard to be
  `required`. Rejected; D1 keeps both tiers.
- **"Is `stem == prefix.lower()` a regression of the POL `stem != prefix` proof?"**
  No — POL's test proved the fields are *independent*; STD is just the aligned
  case. The `GovKind.stem` field still carries its own value.
- **"Any third boot surface beyond `boot_sequence`/`produce`?"** Section ordering
  and projection both flow through those two; the `Active Standards` heading is a
  single `boot_sequence` tuple. No hidden third surface.
- **"Does `default` collide with the supekku `default` semantics elsewhere?"** No —
  it is a per-kind status string, scoped to STD's known-set; ADR/POL never see it.

### 8.1 External adversarial pass (Codex, design + plan)

Six findings, all verified against source and integrated:
- **(MAJOR) Boot "byte-identity proven by suites" overclaimed** — the existing
  boot tests assert presence + row-format + filtering, not full section bytes.
  Reworded §3.5: byte-identity holds *by construction* (provably-unchanged
  inputs); the suites prove semantic preservation. Optional byte golden noted.
- **(MAJOR) install→commit e2e missing** — POL has a dedicated
  `e2e_policy_install_commit.rs` for the gitignore-negation/uncommittable trap;
  the install.rs unit test covers only surface 1. Added VT-5 / §4 parity.
- **(MAJOR) worker-guard gap** — `standard new/status` absent from the
  `e2e_worker_guard.rs` refusal matrix (ADR-006). Added VT-6 / §4 parity.
- **(MAJOR) list parse-conformance** — `e2e_list_conformance.rs` omits even POL;
  "mirror policy" buys nothing. Added VT-7 / §4 (extend matrix to gov kinds).
- **(MAJOR) golden VT-4 vague** — tightened to byte-exact `standard
  list/show/show --json/status` against the ADR golden pattern.
- **(MINOR) scope template fallback stale** — removed the "if exists…else
  policy.md" conditional; supekku source is locked.

## 9. Open questions

None blocking. The boot variant name (`GovRows`) is internal and reversible.
