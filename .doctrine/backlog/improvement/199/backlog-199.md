# IMP-199: Automate R-5 delta-content check against declared file set

**Source:** SL-168 postmortem §5b.4. **Home:** RFC-005 (cross-links RFC-004).

The R-5 import belt rejects `.doctrine/`/`.claude/` touches but is otherwise a
manual orchestrator check; worker scope creep (files outside the declared set) is
only caught by hand.

**Fix direction:** automate — flag any delta file not in the declared design-target
set. Reads the path-intent selectors (RFC-004) as the declared set, so it rides the
selector accretion rather than a bespoke list.

Related: RFC-005, RFC-004.
