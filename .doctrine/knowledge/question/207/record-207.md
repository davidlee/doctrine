# QUE-207: Binary and crate topology for the control plane

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The question

What is the binary and crate story for RFC-025's capsule programme? Specifically,
across four surfaces:

- the **trusted control plane** (provisioning, harvest, quarantine import,
  conformance, admission, canonical Git normalization);
- the **existing CLI** — everything agents and humans already run;
- **provisioning** itself, which straddles both (it runs trusted but writes into
  the capsule);
- **any long-running process** the model implies — a daemon, a broker, the
  existing `doctrine serve` / `map serve` surfaces.

The question underneath all four: **where does the authority boundary get
expressed** — in the environment, in the crate graph, in binary absence, or in a
protocol?

## The frame (external conversation starter, 2026-08-05)

Recorded as posed, not endorsed. Its opening distinction is the useful part:

> **Same product and codebase** is not the same decision as **same crate**,
> **same binary**, or **same authority surface**.

**A — one `doctrine` binary.** Everything stays a subcommand
(`doctrine capsule provision`, `doctrine integrate`, …); privilege comes entirely
from environment, credentials, filesystem access, and control-plane
authentication. Simplest packaging, no IPC for local composition. But the capsule
carries privileged code paths, the distinction between *tool unavailable* and
*tool refused here* becomes load-bearing, and an accidentally exposed socket,
token, Git directory, or config widens authority sharply. Securable if the
control plane independently authenticates every operation — harder to audit and
explain.

**B — one Cargo workspace, separate binaries and authority crates.** Shared
crates (`doctrine-core`, `-protocol`, `-graph`, `-documents`, `-prompts`,
`-capsule`) under two binaries: `doctrine` (user- and agent-facing) and
`doctrine-control` (provisioning, harvest, workflow, canonical Git). The capsule
receives `doctrine`; `doctrine-control` exists only outside it. Both share entity
schemas, contract and result types, selectors and conformance algebra, graph
hydration, evidence formats, normalization policy, protocol clients and servers.
More packaging and interface work, but a real capability boundary without
splitting Doctrine into independently evolving systems. **The frame's provisional
choice.**

**C — separate control-plane system.** A distinct daemon/service/repository,
sharing only versioned protocol definitions or a client library. Strongest
separation and the eventual remote-execution story, but buys distributed-system
and product-boundary costs early: version negotiation, deployment, auth,
compatibility, observability, duplicated lifecycle. Likely premature while the
capsule protocol is still being discovered.

Its provisional direction is **B** under a one-way dependency rule:

```text
agent-facing tools ──► shared pure/domain crates
control plane      ──► shared pure/domain crates
agent-facing tools -X► control-plane implementation
```

The control plane may invoke or embed ordinary Doctrine capabilities; the
ordinary CLI must never import the control-plane implementation. Two further
claims worth testing: that the control binary should **not** be a hidden mode of
the same executable, because *"a separate executable is cheap and lets absence
remain part of the authority model"*; and that a unified user-facing command
(`doctrine capsule status`) is fine so long as it *speaks to* `doctrine-control`
rather than containing the provisioning or canonical-Git implementation.

## What the repo already supplies (do not re-derive)

**A workspace already exists, and B's dependency rule already has a precedent.**
`Cargo.toml` declares `[workspace] members = [".", "crates/cordage"]` — root
package plus one member. `cordage` is documented as a *product-neutral leaf*:
"Doctrine depends on this by path; never the reverse (ADR-001 leaf)". So option B
is not a greenfield restructure; it is another application of a rule the tree
already states and enforces. ADR-001's module layering (leaf ← engine ← command,
no cycles) is the same discipline one level down — the open part is whether the
authority boundary is *the same* boundary as the layering boundary or crosses it.

**The frame does not price distribution, and distribution is where a second
binary actually costs.** Every compile-time embed root must appear in three
places or the artefact ships hollow *with no compile error*:

1. `Cargo.toml`'s `include` allow-list (currently `install/`, `publication/`,
   `plugins/`, `memory/`, `web/map/dist/`, `templates/mcp.ts`);
2. `srcWithDist` in `flake.nix` — `craneLib.cleanCargoSource` filters the sandbox
   to `.rs`/`.toml`/`.lock` only, so any un-grafted embed root is stripped;
3. the release/binstall path — `cargo-binstall` fetches a prebuilt asset
   (SL-174).

A second binary multiplies each of these, and the failure mode is silent. This is
a real cost against B and C that the sketch does not weigh. It also raises a
concrete sub-question: **which embedded assets does the capsule-side binary
need?** If `doctrine` in the capsule needs neither `install/` nor `plugins/` nor
the map, the split may *reduce* the embed surface on the side that matters.

**Workspace-wide tooling already carves out per-crate.** `just check` skips the
cordage workspace crate. A two-binary tree needs that carve-out logic to stay
comprehensible rather than accreting.

**The capsule-side surface is not hypothetical.** Settled direction 2 in RFC-025:
"give each headless capsule worker broad local authority but no control-plane
authority" — that sentence *is* the authority boundary this question asks how to
express. DEC-134 keeps the interactive orchestrator in the trusted control plane
with headless per-phase workers. Settled direction 15 / red-team RT-1 requires
verification to run in a **separate capsule**, never in the trusted control
plane — so there are at least three distinct execution contexts, not two, and a
binary story that only names "inside" and "outside" is underspecified.

**`doctrine serve --mcp` is an unresolved third thing.** It is a long-running
process today, launched from `.mcp.json` as a stdio child of the *harness*, and
on the Claude arm it sits outside every subagent sandbox and resolves paths
against the primary repo root — writable MCP tools bypass confinement entirely
(see IMP-401). Whatever the crate split decides, it has to say where the MCP
server lives and which side's authority it carries. `map serve` is a second
long-running surface with different (read-only, human-facing) properties.

## Considerations the frame omits

- **POL-002** keeps host-project conventions out of the engine. A control-plane
  binary that owns provisioning is closer to host-specific concerns than the
  current CLI is; the split may *help* here by giving those concerns a home
  outside the engine crate.
- **"Absence as authority" is only as strong as provisioning.** The claim that a
  separate executable lets absence do the work holds only if the capsule's
  provisioning manifest reliably omits it — which is exactly RFC-025's open
  next-design-question 2 (*what belongs in the provisioning manifest?*). These
  two questions are coupled and should not be settled independently.
- **A/B is not either/or at the verb level.** `doctrine capsule status` as a thin
  client over a control binary is B; the same string as a subcommand carrying the
  implementation is A. The user-facing grammar can be identical under both — so
  the decision should not be argued from CLI ergonomics.
- **Daemon is a separable axis.** Nothing in RFC-025's v0 requires a persistent
  service: outbound capsule calls are payload-free notifications and the control
  plane inspects authoritative capsule state. A daemon is a C-flavoured cost that
  B does not oblige. Keep the process-model question distinct from the
  binary-count question.
- **Migration cost is asymmetric.** A→B later is a refactor of a tree that
  already has a workspace and a stated one-way rule. B→C later is the extraction
  the frame explicitly leaves open. A→C is the expensive path. That ordering
  favours deciding A-vs-B now and deferring C, which is where the frame lands —
  but for a reason it does not state.

## What would settle it

1. Enumerate the capsule-side verb set concretely — what must `doctrine` do
   *inside* a capsule? If that set is small and disjoint from provisioning,
   harvest, and admission, B is cheap and A's "tool refused here" burden is
   avoidable. If it overlaps heavily, B buys less than it costs.
2. Price the distribution multiplier honestly against the three-place embed
   discipline above, including whether the capsule binary needs any embedded
   assets at all.
3. Settle it jointly with RFC-025 next-design-question 2 (provisioning manifest),
   since "absence is authority" depends entirely on that answer.
4. Name where `doctrine serve --mcp` and `map serve` land under each option.

Answering "A, and privilege stays environmental" is a legitimate outcome —
record it with the audit burden it implies rather than treating B as the default
because it sounds safer.

## Related

- RFC-025 — the capsule programme; this is a new entry in its next-design-questions
  list.
- ADR-020 — capsules as the dispatch authority boundary; ADR-001 — module
  layering, the precedent for the one-way rule.
- DEC-134 — v0 topology (persistent trusted orchestrator, headless per-phase
  workers). DEC-135 — bundle transport and the no-trusted-Git-in-hostile-repo
  invariant, which constrains what the control-plane binary must contain.
- POL-002 — platform independence from host-project conventions.
- IMP-401 — the MCP server's current position outside every subagent sandbox.
