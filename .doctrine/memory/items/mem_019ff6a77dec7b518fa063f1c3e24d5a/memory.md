# Check a repair against the class statement it came from

A repair derived from a sentence that enumerates a class tends to close one
member of that enumeration and leave the rest — and because the class statement
is right there in the artefact, nobody re-reads it.

Evidence: SL-253 design, RV-354, three verification rounds.

`F-2` is the clean case. Section 5.1 names three vacuity modes for the compile
probe **in a single sentence** — a wrong `#[path]`, a target outside the default
selection, and a `required-features` gate left on. Two consecutive repairs, each
correct and each verified as far as it went, closed the first and the third. The
second survived both rounds untouched, three sentences below its own statement.

`F-1` is the same shape twice over. The finding asked for two things — that the
row-runner stop carrying authority out, and that the kernel's callback protocol
be pinned. Three rounds repaired the first and never touched the second. And the
class *generalisation* written to close it was itself wrong twice, both times by
attaching an "only where" to a test that could not carry it — narrowing the
class on each attempt rather than widening it.

## The move

Before treating a repair as complete, find the sentence in the artefact the
repair was derived from — the one naming the class, listing the modes, or
enumerating the requirement. It is almost always within a paragraph of the edit.
Then check the fix against **every member it enumerates**, not the member the
reviewer happened to cite.

This is cheaper than a sibling sweep because the checklist is already written;
the work is reading it back rather than reconstructing it. Prefer it over
recall — the author who wrote the enumeration is the same one who will
misremember its length.

Corollary, learned the same way: a repair narrows the claim it repairs more
often than it widens it. When a repair restates a general rule, suspect the
restatement of having been fitted to the instance that prompted it.

Sibling: [[mem.pattern.review.sweep-defect-class-not-instance]] is the outward
form — sweep the codebase for siblings of a defect class. This one is the inward
form — sweep the *document's own class statement* before looking anywhere else.
