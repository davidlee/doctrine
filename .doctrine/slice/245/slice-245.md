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
- **Env-sniffing would be wrong.** Inside the jail `TERM=xterm-256color` with
  no `KITTY_WINDOW_ID`, so inference would report "no kitty" against a
  terminal that speaks it. Preflight concluded no detection was needed; design
  review (RV-368 F-1) reversed that under POL-002 facet (3), and support is now
  asked of the terminal directly (DEC-259).

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
3. **Descriptive refusals, one error class** (DEC-254, DEC-259, DEC-256): the
   format is not DOT, stdout is not a terminal, the terminal does not report
   its pixel size, or the terminal does not answer the kitty support query
   (tmux included). Exit non-zero with cause and fix on stderr. A failed `dot`
   spawn is reported by its typed outcome: unavailable, failed with graphviz's
   stderr, timed out, or other I/O (DEC-255). Every failure before the image is
   written leaves stdout empty; nothing fails silently.
4. **Typed capability seams** (DEC-255, DEC-259): a `RenderTarget` window
   descriptor and a raw-mode query/reply exchange in `src/tty.rs`; the kitty
   support query plus DA1, classified purely; and a sync `dot -Tpng` spawn
   that is its own availability probe. The spawn is a second, knowingly-held
   `dot` render spawn beside `map_server`'s, each naming the other. The
   program name is single-sourced across all three `dot` invocations, and
   each timeout is named where it is enforced (DEC-143). The spawn rides a
   bounded sync-subprocess helper extracted from `coverage_verify` rather than
   copying it; that suite is the behaviour-preservation proof.
5. **Explicit placement** (DEC-256, provisional): always send columns and rows
   with `C=1` and write the rows as newlines. Native size when it fits in
   columns minus one, else scale to columns minus one.
6. **Both Rust verbs wired** (DEC-258): `doctrine graph` first, proving the
   seam end to end, then `concept-map export`.
7. **Verification** (DEC-257): VH (the user in ghostty, small and large graphs,
   a pipe, tmux) is the acceptance test. VT scaffolds arrival and guards
   against silent regression without graphviz or a terminal: pure tests over
   synthetic inputs, two real spawns, and three CLI wiring tests.
8. **Reconcile-time governance** (DEC-258): a Revision amends SPEC-027 resp. 5
   and REQ-396's `run_graph` acceptance criterion to describe the render
   branch. ISS-242 is annotated that the flag joins the ungoverned concept-map
   surface.

## Non-Goals

- **Relaxing SPEC-027 resp. 4.** Siting the renderer outside it is the whole
  point; if the design finds itself arguing to relax the no-external-renderer
  clause on `catalog::dot::render`, that is a signal the siting is wrong. (The
  resp. 5 note on the verb shell's output mode is descriptive, not a
  relaxation.)
- **Unifying the two `dot` render spawns** — held knowingly per DEC-143.
- **Multiplexer passthrough.** Under tmux the escape sequence needs wrapping.
  Out of scope; the support probe refuses there rather than drawing nothing.
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
