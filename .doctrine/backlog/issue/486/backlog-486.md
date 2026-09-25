# ISS-486: Review prose args are shell-expanded into the committed ledger (env-dump class)

`review dispose --response` and `review raise --detail` take arbitrary prose on the
command line. Unescaped backticks ran shell command substitution and spliced a full
environment dump (5 live API keys) into the authored, committed ledger
(obs `019fc0bc`); `$` expansion does the same to `--detail` (obs `019fd757`).

Fix: accept prose from a file/stdin (`IMP-377` is the dispose half), and treat the
class as a security defect rather than a footgun. See RFC-032 `research.md` F3.
