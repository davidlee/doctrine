# Deferred-but-needed-later items must be captured in a slice or backlog before closeout

User policy: when work is scoped down by deferring something that is *needed
later* (a follow-on skill, a future kind, an unbuilt funnel/seam, a stubbed
integration), that deferral must be represented as a **slice or a backlog entry
before the slice closes**. It may not live only in prose, an ADR consequence
note, or conversation.

**Why:** a "future seam, noted not solved" buried in an ADR or notes is invisible
to `backlog list` and never resurfaces at the start of the next task. The routing
gate consults the backlog, not prose asides — so an uncaptured deferral is a
silently dropped obligation.

**How to apply:** at `/audit`/`/close`, enumerate every conscious deferral the
slice introduced and confirm each has a `backlog new` item (or a successor slice).
A deferral named in an ADR's Consequences/Neutral section still needs its own
backlog row. Distinguish from `/notes` harvest (durable knowledge) — this is about
durable *work* intent. See [[mem.concept.backlog.work-intake-membership]] and the
routing gate's "latent work intent → backlog new" rule.
