# git.rs remote mutations + bare-repo test substrate (SL-148)

Until SL-148, `src/git.rs` was **local-ref-only** (`update_ref_cas`, `resolve_ref`,
`commit_tree`, `merge_base`, `select_remote` — no `push`/`fetch`). SL-148 added
doctrine's **first remote git mutations**, the foundation any future
remote-coordination work builds on:

- `push_ref_cas(root, remote, refname, new_oid, expected_old)` — push **by oid** of
  a dangling commit under `--force-with-lease=<ref>:<expected_old>`; classifies via
  `git push --porcelain` (machine-stable): **only** the explicit lease/create-CAS
  rejection → `RefCas::Moved`; transport/auth/hook/namespace-policy → hard error,
  never a silent retry. Use the zero-oid expected-old for a create-CAS.
- `fetch_refspec(root, remote, refspec)` — explicit per-command refspec; **never
  mutates `.git/config`**.
- `for_each_ref(root, pattern)` — `(refname, oid, author, date, msg)` rows
  (`splitn(5, '\t')`, subject not split).
- Supporting: `commit_empty_tree_as` (dangling empty-tree commit with
  `GIT_AUTHOR_*`/`GIT_COMMITTER_*` set explicitly from the resolved holder, not
  ambient git config), `resolve_holder`, `resolve_remote` (non-repo → `Ok(None)`),
  `EMPTY_TREE_OID`.

**Testing distributed git is jail-safe via a local bare repo, never a network
remote** (the bubblewrap jail blocks network `push`/`fetch`). The substrate:
`git init --bare` temp + N working clones referenced by **explicit path** (so
`.git/config` is never touched), two clones racing the same id to prove
collision-freedom. Network e2e is a manual dev affordance, never a CI dependency.

First consumer: the `GitRef` reservation backend
([[mem.system.engine.identity-claim-seam]]) — `refs/doctrine/reservation/<prefix>/<NNN>`.
