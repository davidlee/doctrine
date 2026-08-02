## The gotcha

`# shellcheck source=/dev/null` is the reflex way to quiet SC1091 ("not
following"). It does quiet it — by telling shellcheck the sourced file is
*empty*. Every symbol the source really publishes then becomes invisible, and
shellcheck starts inventing warnings about them:

- a variable the sourced file **assigns** and this file **reads** → **SC2153**
  ("possible misspelling", often suggesting an unrelated nearby name)
- a variable this file **assigns** for the sourced file to **read** → **SC2034**
  ("appears unused")

Both are false, and both tempt you into `# shellcheck disable=` lines — which
permanently blind the file to the real version of that check.

## The fix

Point the directive at the actual path. `-x` then follows it and the warnings
evaporate on their own:

```sh
# shellcheck source=/abs/path/to/lib/instantiations.sh
. "${RIG}/lib/instantiations.sh"
```

A literal path is required — shellcheck resolves `source=` at lint time and
cannot expand a runtime variable like `${RIG}`. Relative-to-the-script paths
work via `source-path=`, or `SCRIPTDIR` as a placeholder.

## Why it bites here

The spike-capsule rig is a set of sourced shell libraries that deliberately
**publish arrays** because an array cannot survive a `$( … )` — `c3_h5_paths`,
`c3_h9_paths`, `c3_h12_paths` all set `C3_*_PATHS` for their caller. That is
exactly the shape `source=/dev/null` cannot see. The rig's own files lint clean
without any suppression; only drivers that stubbed the source hit it.

Diagnostic tell: the warning names a variable you can `grep` and find assigned
in the sourced file. That is not a misspelling — it is an unfollowed source.

See [[mem.fact.tooling.x-bit-is-not-runnability]] for the sibling lesson that a
cheap check standing in for a real one is where these defects live.
