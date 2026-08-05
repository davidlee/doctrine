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

Three surfaces emit DOT today or will: `doctrine graph` (SPEC-027),
`concept-map export`, and the spec anchor report once IMP-385 lands. IDE-046 is
explicit that this should be **one mechanism those verbs opt into, not three
copies**.

### What preflight settled

- **`dot` is present** in the jail (graphviz 15.1.0), and shelling out to an
  external binary is well-precedented in the codebase (`install.rs` spawns
  `claude`; `coverage_verify.rs` spawns a configured program).
- **`src/tty.rs` already owns the isatty seam** — a pure/impure split that
  probes the terminal in a thin shell and injects plain values into pure
  functions, precisely so goldens stay deterministic. Pipe safety rides this;
  it is not new machinery.
- **`--color auto|always|never`** on both `graph` and `concept-map export` is
  the house precedent for a terminal-capability flag.
- **POL-002 does not bite, and does not bless.** It forbids load-bearing on a
  host project's *conventions* and *transient local state*; a terminal protocol
  and a graphviz binary are neither. Host *tools* are simply unpoliced — see
  Non-Goals.
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
presentation-neutral, and the one-mechanism-three-consumers shape IDE-046 asks
for falls out for free — all three surfaces already produce DOT strings.

## Scope & Objectives

1. **A renderer component sited outside SPEC-027** — DOT text in, terminal
   graphics bytes out. Owned by doctrine, consumed by the emitting verbs rather
   than owned by any one of them.
2. **Opt-in activation.** Rendering never displaces the existing DOT output by
   default; the current behaviour of every verb is unchanged when the feature
   is not asked for. Exact CLI semantics — a separate verb, a `--render` axis
   composing with `--format`, or a `--kitty` shorthand — are a design decision,
   deliberately left open here.
3. **Two guards, both descriptive on failure**: stdout is not a terminal, and
   `dot` is not resolvable. Neither may fail silently or corrupt redirected
   output.
4. **An injected rasteriser.** The `dot` invocation is a thin-shell input to a
   pure encoder, not a subprocess call inside the tested unit — forced, not
   stylistic, because `dot -Tpng` is not byte-stable across graphviz versions
   and goldens can only sit on the framing and encoding layer.
5. **At least one consuming verb wired**, proving the seam works from a real
   caller.

### Left to design

- CLI semantics — separate verb vs. flag axes vs. `--kitty` shorthand.
- Behaviour when the flag is set and stdout is not a tty: hard error, or DOT
  with a note on stderr. The stated bar is that failure be well-handled and
  descriptive; a user who asked for a picture and silently got DOT in a file
  has been surprised.
- Image sizing. The protocol wants pixel or cell dimensions; `src/tty.rs`
  reports width in *cells*, so this needs either the ioctl's pixel fields or
  delegation to the terminal's own scaling.
- Whether SPEC-027 wants a sentence noting a peer renders its output.

## Non-Goals

- **Revising SPEC-027's responsibilities.** Siting the renderer outside it is
  the whole point; if the design finds itself arguing to relax the
  no-external-renderer clause, that is a signal the siting is wrong.
- **Capability handshake.** No kitty query-response round-trip — it is an
  interactive exchange that can hang on terminals that never answer, and opt-in
  makes it unnecessary.
- **Multiplexer passthrough.** Under tmux the escape sequence needs wrapping or
  is swallowed. Out of scope, named rather than omitted.
- **Sixel or other protocols**, and any bundled raster viewer.
- **The POL-002 amendment**, which travels separately: the host-capability
  principle is general governance, stated independently of this feature, and
  should not be smuggled in as one slice's design note.
- **IMP-385** — the anchor report's DOT rendering is a downstream consumer that
  inherits this seam, not part of it.

## Summary

## Follow-Ups
