- pi v0.79.6 ships exactly four built-in tools: `read`, `write`, `edit`, `bash`
- `grep`, `find`, and `ls` are NOT default — they require the `--tools` flag
- Verify: `pi --help` lists the default tool set
- When designing pi-integration spawn templates, explicitly pass `--tools read,bash,edit,write,grep,find,ls` — do not assume they are included
- Source: RV-090 (inquisition of SL-108 design)

See also: [[mem_019eddef5e697f62985341d9b4563862]]
