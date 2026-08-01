# `IFS=$'\t' read` collapses empty TSV fields

The obvious way to read a tab-separated file in bash is wrong whenever a column
can be empty:

```bash
while IFS=$'\t' read -r a b c d; do …; done < file.tsv
```

**Tab is IFS *whitespace*.** POSIX word splitting treats a *run* of IFS
whitespace as a single delimiter, and strips it at both ends. So an empty
interior column disappears and every column after it shifts left:

```
a<TAB>b<TAB><TAB>c<TAB>   →   a=[a] b=[b] c=[c] d=[] 
                              (wanted: a b '' c)
```

This is not a quoting problem and `read -r` does not help — it is the splitting
rule itself.

## The fix

Translate to a **non-whitespace** separator, then split on that. `\x1f` (ASCII
unit separator) is the conventional choice and cannot occur in text data:

```bash
FS=$'\x1f'
while IFS= read -r line; do
  IFS="${FS}" read -r a b c d <<<"${line//$'\t'/${FS}}"
  …
done < file.tsv
```

With a non-whitespace IFS, each delimiter produces exactly one field boundary,
so empty columns survive — including a trailing one.

`awk -F'\t'` is already correct and needs no workaround; this bites only the
bash `read` path.

## Why it is worth a memory

The failure is **silent and mis-attributing**. Observed in SL-241 PHASE-05 T2: a
validator reading an authored spec file reported two violations the file did not
have, because a `dissolution` row's empty `expected-token` collapsed and the
next column's `n/a` landed in it. The validator accused the artefact of the
reader's defect — and the natural repair is to edit the *file* until the reader
stops complaining, which entrenches the bug.

Any results/spec table with optional columns is exposed. If a shell reader of a
TSV reports something implausible about a specific row, check the split before
believing it.
