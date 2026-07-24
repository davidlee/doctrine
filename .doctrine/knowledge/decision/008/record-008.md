# DEC-008: Rust DOT emitter ports both style tables; transparent bg

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->
SL-226 Q2: the Rust DOT emitter ports both dot.ts style tables (per-kind node
fill/font/shape; per-label edge colours) as named-constant tables (STD-001),
and keeps `bgcolor="transparent"` matching the web emitter (user call —
overriding the WCAG dark-bg concern from mem_019ecf333d; consumers choose
their canvas). The tables are independent presentation policy: no parity
contract with web/map/src/dot.ts; divergence is acceptable (R1 mitigation by
declaration, not synchronization).
