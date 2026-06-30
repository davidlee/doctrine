# Capability-as-data at a backend fork point

When factoring a single fork point between platform/backend implementations
(a `Jailer`/strategy trait), route the *selection* as **data resolved by the
shell**, not a zero-arg host lookup inside the pure layer.

**Smell:** `fn select_backend() -> Option<Box<dyn Backend>>` with no args. To
pick with no inputs it must read ambient host state (OS, binary presence) — that
is host detection inside the pure leaf, violating the pure/imperative split
(AGENTS.md: "no clock/git/disk in the pure layer — pass them in as inputs").

**Shape instead:**
- Shell (impure boundary) probes the host → builds a descriptor (pure data):
  `enum Backend { A, B, Deny { reason } }`.
- Pure core takes `&Backend`; `select(&Backend)` is a pure map; the decision fn
  denies with the descriptor's per-arm `reason`.

**Two payoffs beyond purity:**
1. **Make it multi-valued, not a bare `Option`.** A bare `Option`/`bool`
   collapses *absent / unsupported / present-but-degraded* into one value. If any
   downstream arm needs "present but probed-as-unusable ⇒ deny" (e.g. a sandbox
   that exists but refuses to nest), reserve it as a `Deny{reason}` variant now —
   else that arm must widen the type later and refactor the very seam you froze.
2. **Testability.** A pure map over an injected descriptor runs on any CI host —
   you can exercise the "platform X ⇒ deny" arm on a Linux runner with no X
   present. A zero-arg host lookup can only test the host it runs on.

The per-arm `reason` riding the descriptor also keeps actionable error strings
(`"bwrap-unavailable"`) instead of flattening to a generic one.

Born: SL-182 design §5.2, RV-202 (codex GPT-5.5 pass). Sibling of
[[mem.pattern.safety.resolve-every-ref-before-pure-compare]] (shell resolves,
pure compares) and [[mem.pattern.entity.kind-is-data-not-trait]] (prefer data
over an abstracted seam).
