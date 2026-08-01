# IMP-374: Authoring rule gains a delivery-moment clause

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

DEC-103 (*instruction is delivered at the point of effect*) established a third
clause the shipped authoring rule does not carry. This item delivers it.

## The gap

`install/design-prompts/exploring.toml:8-13` states the rule every runbook is
authored against:

```
THE AUTHORING RULE. Could a project legitimately do this differently?
  Yes -> a runbook step. Overridable, verifier substitutable.
  No  -> an engine invariant, enforced by `apply` / `advance`. Never a step.
```

Two branches, total over its two answers, and **no prose branch**. Yet skills do
retain prose, and SL-233 PHASE-08's sketch invented a third tier to hold it —
unauthorised by the very rule it cites as its frame. DEC-103 supplies what was
missing: prose is not a destination but a failure to locate a delivery moment,
and residue is labelled as such.

## Work

1. Extend the authoring-rule header with the third clause — where an obligation
   is neither an invariant nor cleanly edge-shaped, find the moment it takes
   effect and hang it there; hang it at *every* such moment if there are several;
   only if no moment can be located does it stay prose, recorded as
   unenforced-by-construction.
2. Propagate the amended header to every runbook authored after `exploring.toml`
   — as of DEC-103 that is PHASE-08's edge-2, edge-3 and edge-4 runbooks.
3. Consider whether the clause belongs somewhere more central than a per-runbook
   header comment. Repeating an authoring rule verbatim across N runbook files is
   itself the DRY-versus-delivery tension the rule is about, and the answer is
   not obvious: a header that must be read where the authoring happens may be
   exactly right.

## Why it is not in SL-233

Out of PHASE-08's scope. The phase converts three checklists into runbooks; it
does not re-author the rule those runbooks are written against, and widening it
to do so would put an unreviewed governance edit inside a phase whose design gate
(`EN-2`) has already been sketched and is about to be reviewed without it.

Ordered after SL-233 lands, since step 2 needs the three new runbooks to exist.
