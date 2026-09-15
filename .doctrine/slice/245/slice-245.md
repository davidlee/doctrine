# Inline terminal diagram rendering

## Context

Doctrine emits DOT and stops. Seeing a graph is a two-step ritual the user
performs by hand:

```
doctrine graph SL-243 --depth 1 | dot -T png | viu -
```

IDE-046 proposes doctrine close that loop itself — rasterise the DOT and write
the image straight to the terminal via the kitty graphics protocol, which
ghostty and kitty both speak. The argument is not convenience but altitude: a
DOT stream is something you pipe away and read later, while an inline image is
something you glance at mid-conversation, which is where these verbs are
actually used.

Two Rust surfaces emit DOT today: `doctrine graph` (SPEC-027) and
`concept-map export`. The spec anchor report will be a third once IMP-385 lands;
the web explorer's two TypeScript emitters (`web/map/src/dot.ts`) are out of a
Rust module's reach. IDE-046 is explicit that this should be **one mechanism
those verbs opt into, not a copy per verb**.

### What preflight settled

- **`dot` is present** in the jail (graphviz 15.1.0), and shelling out to an
  external binary is well-precedented in the codebase (`install.rs` spawns
  `claude`; `coverage_verify.rs` spawns a configured program).
- **`src/tty.rs` already owns the isatty seam** — a pure/impure split that
  probes the terminal in a thin shell and injects plain values into pure
  functions, precisely so goldens stay deterministic. Pipe safety rides this;
  it is not new machinery.
- **`--color auto|always|never`** is a *global* flag on the root CLI
  (`src/main.rs`), not a per-verb precedent — corrected by research.
- **POL-002 facet (3)** (added by REV-047 during preflight): a feature-scoped
  host tool such as `dot` must be opt-in, and its absence must fail naming what
  was missing and what would satisfy it.
- **Capability detection is not needed.** Opt-in is the capability assertion.
  Env-sniffing would in fact be wrong: inside the jail `TERM=xterm-256color`
  with no `KITTY_WINDOW_ID`, so inference would report "no kitty" against a
  terminal that speaks it.

### The constraint that shapes this slice

SPEC-027's fourth responsibility requires its DOT emitter to work *"with no
filesystem or external-renderer dependency."* Rendering cannot therefore be
grown inside `doctrine graph`'s component without revising that spec.

**Decision taken before scoping (user, 2026-08-05): site the renderer outside
SPEC-027.** A small owned component consumes *DOT text* and emits terminal
bytes. SPEC-027's disclaimer stays literally true, its projection stays
presentation-neutral, and the one-mechanism shape IDE-046 asks for falls out for
free — every emitting surface already produces DOT strings.

## Scope & Objectives

1. **A renderer component sited outside SPEC-027** — DOT text in, terminal
   graphics bytes out. Owned by doctrine, consumed by the emitting verbs rather
   than owned by any one of them.
2. **Opt-in activation via `--render` / `-X`** on each DOT-emitting verb
   (DEC-253), default off. Every verb's behaviour is unchanged without it. On
   `concept-map export`, `-X` implies `--format dot` (DEC-258).
3. **Descriptive guards, one error class** (DEC-254): stdout is not a
   terminal, or `-X` is combined with a non-DOT format — exit non-zero, nothing
   on stdout, cause and fix on stderr. A failed `dot` spawn is reported by its
   typed outcome — unavailable, failed with graphviz's stderr, or timed out
   (DEC-255). Nothing fails silently or corrupts redirected output.
4. **Typed capability seams** (DEC-255): a `RenderTarget` terminal descriptor
   (`NotTerminal` | `Terminal { columns, pixel_width }`) probed in `src/tty.rs`,
   and a sync `dot -Tpng` spawn that is its own availability probe. A pure
   encoder turns PNG bytes plus the descriptor into kitty escape bytes. The
   spawn is a second, knowingly-held `dot` spawn beside `map_server`'s, each
   naming the other, program name and timeout single-sourced (DEC-143). The
   spawn rides a bounded sync-subprocess helper extracted from
   `coverage_verify` rather than copying it; that suite is the
   behaviour-preservation proof.
5. **Sizing by pixel width** (DEC-256, provisional): native size when the PNG
   fits the terminal's pixel width, else scale to columns; unknown pixel width
   scales to columns.
6. **Both Rust verbs wired** (DEC-258): `doctrine graph` first, proving the
   seam end to end, then `concept-map export`.
7. **Verification** (DEC-257): VH — the user views small and large graphs in
   ghostty — is the acceptance test. VT scaffolds arrival and guards against
   silent regression without graphviz or a terminal: encoder framing goldens
   over a committed PNG fixture, sizing table, descriptor decision, end-to-end
   guard errors, and spawn outcomes with program and timeout injected.
8. **Reconcile-time governance** (DEC-258): a Revision adds a sentence to
   SPEC-027 resp. 5 on the render output mode; ISS-242 is annotated that the
   flag joins the ungoverned concept-map surface.

## Non-Goals

- **Relaxing SPEC-027 resp. 4.** Siting the renderer outside it is the whole
  point; if the design finds itself arguing to relax the no-external-renderer
  clause on `catalog::dot::render`, that is a signal the siting is wrong. (The
  resp. 5 note on the verb shell's output mode is descriptive, not a
  relaxation.)
- **Unifying the two `dot` spawns** — held knowingly per DEC-143.
- **Capability handshake.** No kitty query-response round-trip — it is an
  interactive exchange that can hang on terminals that never answer, and opt-in
  makes it unnecessary.
- **Multiplexer passthrough.** Under tmux the escape sequence needs wrapping or
  is swallowed. Out of scope, named rather than omitted.
- **Sixel or other protocols**, and any bundled raster viewer.
- **The POL-002 amendment** — travelled separately and landed as REV-047;
  this slice consumes facet (3), it does not author it.
- **A force mode** that emits escape bytes to a non-terminal — a separate
  opt-in if ever wanted (DEC-254).
- **The web explorer's TypeScript DOT emitters.**
- **IMP-385** — the anchor report's DOT rendering is a downstream consumer that
  inherits this seam, not part of it.

## Summary

## Follow-Ups
