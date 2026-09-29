libtest's `--report-time` and JSON output are unstable flags. They work on stable with `RUSTC_BOOTSTRAP=1`:

```
RUSTC_BOOTSTRAP=1 cargo test -p doctrine -- -Z unstable-options --format json --report-time > times.jsonl 2> times.err
grep -o '{ *"type": *"\(test\|suite\)".*}' times.jsonl | jq -c . > clean.jsonl
jq -r 'select(.type=="test" and .event=="ok") | "\(.exec_time)\t\(.name)"' clean.jsonl | sort -gr | head
```

The `grep -o` step is required. Child processes spawned by tests write straight to fd 1, which glues stray text onto JSON lines, and a plain `jq` then stops at the first bad line. Suite rows pair with `Running …` lines in stderr, in order.

Duplicate executions: `jq -r '… .name' | sort | uniq -d`.

Baseline on 2026-09-29: 9,039 executions, 2m04s wall (mostly compile), 166 s summed, 11 tests over 1 s (listed in RV-411's brief).
