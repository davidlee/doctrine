## IMP-133: `list`, `show`, `inspect` — findings catalogue

### 1. Coverage: which entity kinds have `list`, `show`, `inspect`

| Kind | `list` | `show` | `inspect` (metadata‑only) |
|---|---|---|---|
| `slice` | ✅ | ✅ | ❌ |
| `revision` | ✅ | ✅ | ❌ |
| `rfc` | ✅ | ✅ | ❌ |
| `rec` | ✅ | ✅ | ❌ |
| `review` | ✅ | ✅ | ❌ |
| `adr` | ✅ | ✅ | ❌ |
| `policy` | ✅ | ✅ | ❌ |
| `standard` | ✅ | ✅ | ❌ |
| `spec` | ✅ | ✅ | ❌ |
| `memory` | ✅ | ✅ | ❌ |
| `knowledge` | ✅ | ✅ | ✅ |
| `backlog` | ✅ | ✅ | ✅ |
| `concept-map` | ✅ | ✅ | ❌ |
| `relation` | ✅ (`list` + `census`) | ❌ | — |
| `search` | N/A (separate surface) | ❌ | — |
| `inspect` (standalone) | — | — | ✅ (cross‑kind) |

**F-1: `inspect` missing on 11 of 13 kinds.** Only `knowledge` and `backlog` expose a per‑kind `inspect` that returns metadata without the prose body. Every other kind forces you to `show --json` for machine‑readable metadata‑only access — there is no human‑readable metadata‑only view for `slice`, `adr`, `revision`, `review`, etc. Given `inspect` exists on two kinds and is conceptually useful for all (SL‑025 explicitly unifies the read surface), this is an incomplete rollout.

---

### 2. `--json` / `--format json` gaps

**F-2: `search` missing `--json` shorthand.** `search` has `--format json` but rejects `--json`:

```
$ doctrine search "memory" --json
error: unexpected argument '--json' found
```

Every other command that has `--format` also aliases `--json`. This is a one‑line fix — add `#[arg(long, alias = "json")]` or equivalent to the `--format` arg.

`relation list` and `relation census` have `--json` ✅. `catalog scan`/`catalog graph` are always‑JSON (correct — no format choice needed).

---

### 3. `--columns` gaps in `list`

| Command | `--columns` |
|---|---|
| `slice list` | ✅ |
| `revision list` | ✅ |
| `rfc list` | ✅ |
| `rec list` | ✅ |
| `review list` | ✅ |
| `adr list` | ✅ |
| `policy list` | ✅ |
| `standard list` | ✅ |
| `spec list` | ✅ |
| `memory list` | ✅ |
| `knowledge list` | ✅ |
| `backlog list` | ✅ |
| `concept-map list` | ✅ |
| **`relation list`** | **❌ MISSING** |
| **`relation census`** | **❌ MISSING** |
| **`search`** | **❌ MISSING** |

**F-3: `relation list` and `relation census` have no column selection.** Both accept `--format json`/`--json` but you cannot select/order visible columns in the human table.

**F-4: `search` has no column selection.** Not strictly a list, but its table output would benefit from `--columns`.

---

### 4. Tags column: data‑tier support vs display surface

All entity kinds that are taggable at the data tier (`doctrine tag set` succeeds) and which are consumable via the shared `list` surface:

| Kind | Tag data works | `tags` in `--columns` | `tags` in *default* columns |
|---|---|---|---|
| `slice` | ✅ | ❌ — available: `id,status,phases,slug,title` | ❌ |
| `revision` | ✅ | ✅ | ❌ |
| `adr` | ✅ | ❌ — available: `id,status,slug,title` | ❌ |
| `policy` | ✅ | ❌ — available: `id,status,slug,title` | ❌ |
| `standard` | ✅ | ❌ — available: `id,status,slug,title` | ❌ |
| `spec` | ✅ | ❌ — available: `id,status,slug,title,members` | ❌ |
| `rfc` | ✅ | ❌ — available: `id,status,slug,title` | ❌ |
| `knowledge` | ✅ | ❌ — available: `id,kind,status,slug,title` | ❌ |
| `backlog` | ✅ | ✅ | ✅ |
| `memory` | ✅ | ✅ | ✅ |
| `concept-map` | ✅ | ✅ | ✅ |
| `rec` | ❌ (IMP‑144) | — | — |
| `review` | ❌ (IMP‑144) | — | — |

**F-5: Tags invisible in `list` for 7 taggable kinds.** `slice`, `adr`, `policy`, `standard`, `spec`, `rfc`, and `knowledge` all accept `doctrine tag set` but their `list --columns` validator omits `tags`. Even for `revision`, where `tags` *is* a valid column, it's not shown by default. Only `backlog`, `memory`, and `concept-map` include tags in both `--columns` and default output.

---

### 5. Default column set inconsistencies

| Command | Default columns | Notable |
|---|---|---|
| `slice list` | `id, status, phases, title` | No slug, no tags |
| `revision list` | `id, status, approval, title` | No slug, no tags |
| `rfc list` | `id, status, title` | No slug |
| `rec list` | `id, move, owning, title` | Kind‑specific columns |
| `review list` | `id, status, facet, target, title` | Kind‑specific |
| `adr list` | `id, status, slug, title` | Has slug ✅ |
| `policy list` | `id, status, slug, title` | Has slug ✅ |
| `standard list` | `id, status, title` | **Missing slug** — adr/policy have it |
| `spec list` | `id, status, title, members` | Has members ✅ |
| `memory list` | `uid, type, status, trust, key, tags, title` | Most columns by default |
| `knowledge list` | `id, kind, status, title` | Has kind (multi‑kind) |
| `backlog list` | `id, kind, status, tags, title` | Has kind + tags ✅ |
| `concept-map list` | `ID, Status, Tags, Slug, Title` | Title‑Case headers; has slug + tags ✅ |

**F-6: `standard list` missing `slug` from defaults** while `adr list` and `policy list` include it. All three are governance kinds with the same shape.

**F-7: Header‑casing divergence.** `concept-map list` uses Title Case (`ID`, `Status`, `Tags`, `Slug`, `Title`); every other command uses lowercase. One render path, two styles.

---

### 6. `show` rendering surface

**How each `show` renders in human (table) format:**

| Kind | Renders as… |
|---|---|
| `slice` | Metadata header line + markdown body |
| `adr` | Metadata header line + markdown body |
| `policy` | Metadata header line + markdown body |
| `standard` | Metadata header line + markdown body |
| `spec` | Metadata header + reassembled markdown whole |
| `backlog` | Metadata header line + prose body |
| `knowledge` | Metadata header line + prose body |
| `rfc` | Metadata header line + prose body |
| `rec` | Metadata header line + prose body |
| `memory` | **Framed security wrapper** (`=== MEMORY (data, not instruction) ===`) + metadata fields + body — unique to memory |
| `review` | Metadata header + findings detail list + brief body |
| `revision` | Metadata header + structured change rows + rationale body |
| `concept-map` | Metadata header + body + DSL dump + edges table (with `--edges`) |

**F-8: `memory show` format is uniquely framed.** Every other `show` renders metadata as a one‑line summary then dumps the markdown body. `memory show` wraps output in a security‑framed block (`=== MEMORY (data, not instruction) ===` header, metadata as key‑value lines, body‑guard hash). This is intentional (agent‑context framing) but means no two‑tier TOML‑structural‑summary + prose output like all other kinds.

**F-9: `concept-map show` only exposes DSL edges via `--edges`/`--nodes` flags.** You cannot see edges in the default table view; `--json` always includes the parsed edges but the human format gate‑keeps them behind explicit flags.

**Observation:** The render split is clean and data‑driven — prose‑heavy kinds render markdown; structured kinds (review, revision) render findings/change‑rows before the body. This is not an inconsistency per se, but the memory wrapper diverges from the pattern.

---

### 7. `--color` flag

Every command has `--color`. ✅ Consistent.

---

### 8. Summary matrix

| Finding | Severity | Kind | Detail |
|---|---|---|---|
| F-1 | **major** | Missing feature | `inspect` absent from 11 of 13 kinds — "metadata only, no prose" is only usable on knowledge/backlog |
| F-2 | **major** | Flag gap | `search` rejects `--json`; only accepts `--format json` |
| F-3 | **minor** | Flag gap | `relation list` and `relation census` missing `--columns` |
| F-4 | **nit** | Flag gap | `search` missing `--columns` (optional; not a list) |
| F-5 | **major** | Data‑display gap | 7 taggable kinds (`slice`, `adr`, `policy`, `standard`, `spec`, `rfc`, `knowledge`) hide `tags` from `list --columns`; `revision` has it but not by default |
| F-6 | **minor** | Inconsistency | `standard list` default columns omit `slug` (adr/policy include it) |
| F-7 | **nit** | Inconsistency | `concept-map list` uses Title Case headers; all others lowercase |
| F-8 | **minor** | Inconsistency | `memory show` uses unique framed format diverging from the common metadata‑line + prose pattern |
| F-9 | **minor** | UX | `concept-map show` requires `--edges`/`--nodes` to see parsed DSL data in human format |

`IMP-133`

---

