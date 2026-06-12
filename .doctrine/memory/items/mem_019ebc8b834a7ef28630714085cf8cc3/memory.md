# comfy-table last-column cell-fill survives zero padding — trim_end each line for no trailing whitespace

Under `ContentArrangement::Disabled`, comfy-table still fills every cell to its
column's measured width (that is the alignment it provides). Zeroing the last
column's right **padding** (`Column::set_padding`) removes the padding char but
NOT this content fill, so a short last-column cell still emits trailing spaces.

To reproduce a hand-rolled renderer's "no trailing whitespace on any line"
property, `trim_end()` every rendered line after `to_string()` (and re-append
`\n`, since comfy-table omits the final newline). Pad-zeroing handles the outer
edge separator; `trim_end` handles the residual cell fill — you need both.

Also load-bearing for byte-stable piped output: `set_content_arrangement(Disabled)`
(never measure the terminal) AND `Table::force_no_tty()` before `to_string()` —
`custom_styling` transitively enables the `tty` feature, whose content formatter
otherwise reads `stdout().is_terminal()` at format time. See
`mem.pattern.render.comfy-table-custom-styling-pulls-tty`.

Landed in `src/listing.rs::render_table` (SL-053 PHASE-01); the gap was caught at
the determinism spike and reconciled into the design at audit (RV-011 F-1).
