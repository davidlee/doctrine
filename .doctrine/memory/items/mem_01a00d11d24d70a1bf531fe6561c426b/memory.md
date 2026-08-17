# A design's call-site census ages — make the compiler recount it

A design or plan criterion that says *"all three call sites of `foo` move to the
new form (`a.rs:2077`, `b.rs:2099`, `c.rs:167`)"* is a **claim about the tree at
authoring time**, in exactly the way
[[mem.pattern.testing.grep-for-the-pin-before-characterising]] says a design's
account of what is *untested* is. Between authoring and execution, earlier
phases move lines and add callers. Both halves rot:

- **the line numbers**, harmlessly and visibly;
- **the count**, silently and consequentially — a criterion that quantifies over
  a population you cannot see is one you cannot discharge.

On SL-238 PHASE-06 the census read three and the tree held four. The missed
caller was in a function a *later* phase rewrites wholesale, so the design's
reasoning ("two of the three are being rewritten anyway") applied to it too and
nobody noticed it was absent from the list.

## The cheap re-derivation

Before planning against such a criterion, grep for the population **with a
positive control** ([[mem.pattern.harness.grep-negative-needs-positive-control]])
and confirm each hit with `Read` rather than quoting grep from working memory.
Amend the criterion in place if it is wrong — id kept, reasoning inline — and log
the design half for reconcile if the design run is locked.

## The stronger move: let the type system do the counting

When the change involves **reshaping a signature** — a new parameter, an enum
argument replacing positional ones — the compiler enumerates every caller for
free and *cannot* miss one. That makes the census self-verifying, and it is a
reason to prefer the reshape over an additive overload that leaves old callers
compiling:

```
remove(path, to, ceiling)  ->  remove(path, &RelRemove)   // 4 errors, exactly 4 callers
```

The grep is what lets you *plan* correctly; the reshape is what *proves* it. Run
the grep anyway — you need the number before you write the plan, not after the
build fails.

## Where this does not help

A census over something the compiler does not check — a string literal, a doc
mention, a test name, a config key — gets no such oracle. There, the grep with a
positive control is the whole method, and
[[mem.pattern.verification.removal-claim-attributes-every-survivor]] applies:
count the producers in the pre-state, and attribute every post-state survivor.
