# ISS-338: gid_map has no off-jail reading

The off-jail payload reports `uid`, `gid` and `uid_map` on every arm — and
`gid_map` on none.

## Why that matters

`gid_map` is the fourth surface, and `D2` ruled it in (`F-41`) **precisely
because** it is the mapping leg that discriminates on a uid-1000 host. On such a
host the uid leg alone cannot tell a confined credential set from an ordinary
one; the gid mapping is what carries the signal. It is unconfirmed on a real
host.

This sits outside the three surfaces `VA-1` names, so the negative path is
untouched and this is not a shortfall against that criterion.

## Why it is recorded rather than shrugged at

An unmeasured surface that everyone assumes moved with its sibling is exactly how
the retired `CredentialsConfined` row happened.

## Bearing on an open reconciliation decision

`SL-248` reconciliation must choose whether to keep `CAPSULE_UID` at 1000 (with
row 13's uid half carried as stated-but-non-discriminating) or declare a capsule
uid no ordinary operator holds. Off-jail arm `A3` is the evidence that the second
works — but with `gid_map` unmeasured on every off-jail arm, that evidence is one
leg short. Worth closing before the choice, not after.

## References

`SL-248` `notes.md` § *Owed* item 175(a) · `VA-1` · `D2` / `F-41` · `RV-352` ·
sibling of `ISS-339`
