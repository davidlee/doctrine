# DEC-371: Memory attestation is claim-shaped; the anchor and --allow-dirty retire

Settles `RFC-035` forks 1–3 (stamp meaning, anchor survival, `--allow-dirty`),
checked against fork 4 (consumers set the ceiling).

## Consumer census (2026-10-07)

Every reader of `verified_sha` passes it as the `since` base of
`git::commits_touching` over a path set; none reads whole-tree cleanliness.

| reader | paths | use |
|---|---|---|
| `retrieve::staleness` / `git_facts` | `scope.paths` | Fresh / Stale / Unknown |
| `memory_health_findings` check 2 | `scope.paths` | "N commits behind HEAD" |
| `memory_health_findings` check 4 | own `memory.md` | own-body drift |
| `show` header | — | `verified yes` iff non-empty |
| retrieve ranking | — | reads `verification_state`, not the sha |

The only whole-tree assumption is a comment in `retrieve::staleness`
("verify refuses a dirty tree, so the SHA is always clean"), which no code
depends on.

## The overloaded stamp

`stamp_verification` writes `checkout_state_id` (sha256) into `verified_sha`
under `--allow-dirty`. Corpus at decision time: 1232 empty, 48 commit shas,
62 `checkout_state_id`s. For those 62, `commits_touching` fails the ancestry
guard and returns `None`: `retrieve` renders Unknown (correct), `validate`
skips silently (`ISS-257`, contrary to `STD-003`), `show` and ranking treat
them as verified. Hence the one-shot reset.
