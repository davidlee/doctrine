# Review RV-252 — design of SL-203

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Inquisition on the SL-203 design (facet `design`, raiser `inquisitor`). Two
adversaries: this Inquisitor and an external pass by codex (GPT-5.5). SL-203 is
a **precedent-setting** warm-up — it establishes the cycle-breaking pattern
SL-204 will follow — so the design's rationale and verification must survive
challenge, not merely its happy path.

**Subject.** Dissolve the `{commands, mcp_server}` 2-node dependency SCC by
severing the incidental back edge `mcp_server::tools::render_model_band_guidance
→ commands::prompt::model_keys` (`tools.rs:1365`) via fn-pointer injection (D-B)
through `McpConfig`; forward edge `serve → mcp_server::serve` unchanged.

**Doctrine held to.** ADR-001 (leaf←engine←command, monotone tangle ratchet);
behaviour-preservation gate (AGENTS.md); STD-001 (single-source named
constants); RSK-227 (gate blind to intra-tier concentration → prefer zero new
same-tier edge); `mem.pattern.lint.back-edge-tangle-inject-fnptr` (the blessed
pattern); `mem.pattern.lint.module-split-needs-layering-entry` (extraction's
gate cost).

**Lines of interrogation.**
- §7 D-A vs D-B — is "adds no new same-tier edge" airtight, and is D-B strictly
  better than D-A on the RSK-227 concentration axis, or overstated? *(held)*
- §9 VT-1 −2 ratchet (123→121) — is deferring empirical SCC-membership
  measurement to execute-time acceptable at design lock, or must it be measured
  now? *(mostly held)*
- §5.5 cfg(test) exclusion — sound for **all** test callers, or does one escape
  the `#[cfg(test)]` prune? And does the design's enumeration of the edit
  surface match reality?
- §9 VT-3 wiring guard — does a mis-wired `ModelKeysFn` really escape VT-1, and
  is the proposed guard sufficient to catch it?
- §8 R3 / STD-001 — is invoking STD-001 on the `120` header comment correct?

**Convergent verdict (both adversaries).** The core §7 decision (D-B over D-A)
**survives**. Three defects taint verification/remediation: VT-3 is an
ineffective guard (blocker), R3/STD-001 is misapplied (major), §5.5 understates
the edit surface (major). Charges raised below.

## Synthesis

> *Heresis uritor; doctrina manet.* The bones of this design are sound — but its
> verification wore false vestments, and the Inquisition stripped them bare.

**Judgement.** The load-bearing decision — **D-B fn-pointer injection over D-A
extraction (§7)** — is **NOT heresy**. Two adversaries, this Inquisitor and the
external interrogator (codex GPT-5.5), pressed it independently and it held: D-B
adds **no new same-tier inter-unit edge** the layering gate would count (it only
strengthens the pre-existing `commands → mcp_server` edge, intra-unit, gate-
invisible), and it is **strictly better than D-A** on the RSK-227 concentration
axis — extraction merely relocates the upward reach to `mcp_server → install`
(`model_keys`'s gather is anchored to the RustEmbed `Assets` command tier,
`prompt.rs:279-282`), feeding the very concentration the risk warns of. The
precedent SL-204 will inherit is therefore doctrinally clean.

But the **verification and remediation confessed their sins under
cross-examination.** Four charges, all reconciled into `design.md` before this
design is permitted to advance:

- **F-1 · blocker · VT-3 was inert.** The wiring guard asserted the model-band
  section merely "present and non-empty" — yet `render_model_band_guidance`
  (`tools.rs:1366-1375`) renders the placeholder `(no model keys in corpus)`
  under an always-present header, so an **empty or wrong producer would pass the
  guard untouched**. A guard that cannot fail guards nothing; it would have let
  a silent mis-wire ride to production wearing the mask of a green test. Penance:
  VT-3 now asserts a **known corpus key renders** as content; the "null fn-ptr"
  risk (impossible in safe Rust) is re-cast as a wrong same-signature producer;
  the sole production supply site (`run_serve`, `serve.rs:28`) is named as
  resting on compile-time coercion + VA-1, not VT-3.
- **F-2 · major · STD-001 invoked against history.** R3 would have "fixed" the
  `120` in `architecture_layering.rs` (lines 8 **and** 22 — the design saw only
  one) to `121`. But that `120` is a **historical SL-112 `PHASE-01 go/no-go`
  report snapshot**, not a live constant — the true baseline single-sources at
  `layering.toml:156` and ratchets `123→121` on its own. Overwriting a dated
  report to a value that is neither its historical figure nor sourced is a
  corruption dressed as hygiene. Penance: the report block is left untouched; R3
  reframed as a non-goal; the §3 STD-001 force corrected.
- **F-3 · major · the edit surface was undercounted.** §5.5 confessed to four
  `#[cfg(test)]` `dispatch` callers; the truth is **~13 direct call sites plus
  the `memory_dispatch` helper** (`tools.rs:1900`), all within one `#[cfg(test)]
  mod tests` (`:1386`). The gate-exclusion claim itself **holds** (the extractor
  prunes cfg(test) mod bodies), so no correctness breach — but an understated
  surface misleads `/plan` and turns a missed call site into a mid-phase compile
  failure. Penance: §5.5 and the §9 touch-set now state the full ~14-signature
  surface.
- **F-4 · minor · VT-1 overclaimed its evidence.** The standard gate proves the
  **scalar ratchet** `count_tangle_edges(command) ≤ 121`, not SCC *membership*;
  "Tarjan no longer groups `{commands, mcp_server}`" is inferred from the −2
  drop or shown via the `#[ignore] dump_real_graph` diagnostic. Deferring the
  empirical −2 to execute-time is **acceptable at design lock** (both adversaries
  concur — RSK-227 already grounds the split; the sole production back-edge is
  `tools.rs:1365`). Penance: VT-1 now separates what the gate proves from what is
  inferred.

**Corrective sequence (for `/plan`).**
1. Rewrite VT-3 to assert real key content (red/green), and cover/annotate the
   `run_serve` supply site — F-1's penance is the load-bearing one.
2. Thread the `ModelKeysFn` param through **all ~13 cfg(test) `dispatch` callers
   + the `memory_dispatch` helper**, not merely four (§5.5).
3. Ratchet `layering.toml:156` `command = 123 → 121`; **do not** touch the
   `120` historical report prose in `architecture_layering.rs`.
4. Verify the −2 empirically at execute-time (VT-1); optionally cite
   `dump_real_graph` for SCC-membership evidence.

**Standing risks (consciously carried, none blocking).** The layering gate
remains **blind to intra-unit concentration** (RSK-227) — D-B strengthens the
existing `commands → mcp_server` edge and adds threaded coupling the scalar
ratchet cannot see. This is the accepted residual of the whole strategy, not a
defect of SL-203; captured under RSK-227, deferred by design (§Non-Goals).

**Verdict: the design is fit to advance to `/plan`** once the four penances
above are carried into execution — which they now are, in canon. The heresy
burned; the doctrine stands.
