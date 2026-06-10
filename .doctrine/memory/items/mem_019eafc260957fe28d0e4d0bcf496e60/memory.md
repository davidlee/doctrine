# Entity-engine Kind is data, not a trait — verb seam is intentionally not abstracted

The verb→engine boundary in `src/entity.rs` is deliberately leaky. `Kind` is a
data descriptor — `{ dir, prefix, scaffold: fn(&ScaffoldCtx) -> Result<Fileset> }`
— a struct with a function pointer, NOT a trait. Verb modules (slice, adr, spec,
memory, backlog, …) construct a `const Kind` value and hand it to
`entity::materialise(...)`; they do not `impl` any entity trait. The comment in
entity.rs states it outright: "A Kind is data, not a trait."

**Why (the rationale, so a future agent does not "helpfully" abstract it):** ~8
kinds, all known at compile time, no plugin story. Trait-object polymorphism
would buy dispatch flexibility that is never exercised, at the cost of a vtable
indirection and a flow that is harder to grep. Data-driven dispatch keeps the
whole materialisation path one `grep` away and the dispatch site singular. The
`GovKind` wrapper (SL-033) extends the SAME pattern — per-kind data
(`{ kind, stem, statuses, hidden }`) collapsing adr/policy/standard onto one
governance code path.

**Accepted tradeoff:** a new kind must know engine internals (call `materialise`
directly, construct a `Kind`); there is no trait to "fill in". Fine for a
single-author tool. The seam that WOULD need a trait is third-party / plugin
kinds — if that story ever arrives, this is the boundary to revisit. Until then:
do not trait-ify it.

Related: the only abstract seam in the engine is `Claim` (swappable mkdir-claim
backend), and in practice `LocalFs` is always used.
