Under `reach = local` (and `auto` degraded to local), a fresh-id claim is a zero-oid create CAS (`git::update_ref_cas`) on `refs/doctrine/reservation-local/<PREFIX>/<NNN>` in the clone's shared ref store, pointing at a holder-stamped empty commit (`git::commit_empty_tree_as`), then the per-tree mkdir. A non-git root keeps plain mkdir.

The candidate scan unions: the tree's own entity dirs, trunk ids (ADR-006 D3), the entity dirs of every live worktree (`git worktree list`; covers pre-D9 mints and hand-made dirs in live trees), and both `reservation-local/<PREFIX>/` and `reservation/<PREFIX>/` refs, each read prefix-scoped (ISS-221).

`local` changes meaning from 'this tree' to 'this clone'; no new reach value. `shared` is unchanged. Local claims are never pushed on a local→shared switch; scanning both namespaces keeps mixed modes collision-free.

Residual accepted: a pre-D9 entity committed only on a branch with no live worktree; git merge is the backstop.

Rejected: a new `reach = clone` value (keeps the per-tree footgun selectable and the default for pinned configs); scanning local branch heads (misses uncommitted dirs in a live coord tree — the ISS-279 case); placeholder dirs in other trees (ISS-279: breaks doctor, misdirects citations). Source: RFC-032 D9.