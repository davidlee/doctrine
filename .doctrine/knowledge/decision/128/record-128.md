# DEC-128: QUE-200's residual is architectural, not evidential

The trap this record exists to prevent is a well-meaning one: reading QUE-200's
`open` status as unfinished business and commissioning another probe round to
close it.

QUE-200 asks for the **minimal safe** parent-side mechanism for ingesting a
phase result from a hostile capsule repository. SL-241 measured sixteen hazard
rows against both candidate mechanisms on two fixtures each, and the mechanism
axis turned out to barely matter. That is a real result. It is not an answer to
the question.

## Why more evidence of the same kind cannot help

The unmeasured surface is M-A's: a plain-path `git fetch` spawns
`git-upload-pack '<path>'` **in the capsule repository's own context**. QUE-200's
dissolution reasoning — *"config and hooks are repo-local, never objects"* —
describes what **travels**, and is silent about the trusted side **going to** the
hostile config.

The probe sampled that surface at two keys. `uploadpack.packObjectsHook` was
planted deliberately and git's protected-config defence held. `core.fsmonitor`
**is** honoured from repo-level config and stayed silent only because nothing in
the M-A harvest path refreshes an index — `git status` fires it, `rev-parse` does
not. That safety is a property of *which commands the harvest script happens to
run*, not a property of git.

So the claim QUE-200 needs is a **universal** over git's configuration space:
*protected-config covers everything a hostile repo config can do to upload-pack*.
Two keys do not discharge a universal, and neither do twenty. The result is also
bound to git 2.54.0 — a version bump reopens even the sampled part.

## What would actually close it

Not more sampling. A structural change to the question:

> Does the parent need to run git inside the untrusted repository at all?

M-B runs git in the capsule repo **zero** times; M-A runs it **three**
(`rev-parse` ×2, `fetch`). That is the axis with the leverage, and it is a design
question, not a measurement.

Note the shape of the trade this sits in: M-B's own asymmetry — needing four
trusted-side refusal legs to parse a capsule-authored file — is a **cost**, and
costs are enumerable and testable. M-A's is a **safety** asymmetry against a
surface that cannot be enumerated. The question asks for the minimal *safe*
mechanism, and *"M-B has costs"* is not *"M-A is safe."*

## Reading this alongside the corpus

`evidence/README.md` limit 1 carries the sampled-not-cleared boundary and was
corrected in place during PHASE-06 rather than left for reconcile — the go/no-go
would otherwise have cited a limit its own author knew to be false. See
[[EVD-010]] for the disputed-advantage finding and [[EVD-008]] for why
candidate 3 never entered.
