A falsifiability round often needs a mutant that **reorders** the subject's
internal steps — "what if this ran before that?" The obvious move is to copy the
function and swap two lines. That copy is then the thing under test, and it
drifts from the real one the moment either changes.

**Compose around it instead.** Perform the later step yourself in the wrapper,
then call through to the unmutated original. Its own earlier step now runs
against the state the later one already produced, which is the forbidden
ordering — reached without owning a line of the body.

    # subject: precondition -> transfer -> compare-and-swap
    advance_stage() {
      local canonical=$1 q=$2
      # the transfer, hoisted ahead of the real precondition
      git -C "${canonical}" fetch --no-tags --quiet -- "${q}" "${REF}" || true
      real_advance_stage "$@"          # rebind'd original, untouched
    }

**Why it matters beyond tidiness.** The mutant's red is only evidence about the
real function if the real function is what ran. A restated body proves the copy
was inverted; a wrapper proves the subject was.

**It does not reach every ordering.** Composition can hoist a step earlier or
add one; it cannot delete or reorder steps that are purely interior with no
observable seam. When that is what you need, prefer changing the *world* the
function runs in over forking the function — a call-count-gated no-op on a
collaborator (suppress the second call, keep the first) reaches a lot of cases a
body edit would otherwise be reached for.

**Pair the red with an isolation control that names what still HELD.** The
finding usually lives there, not in the red: if an inverted ordering reds only
one clause and leaves the others standing, that tells you which clause is
load-bearing — and that a weaker assertion would have scored the defect green.

See [[mem.pattern.tests.mutate-the-data-not-just-delete-it]] for the sibling
rule about what to perturb.
