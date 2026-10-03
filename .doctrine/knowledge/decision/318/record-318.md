# DEC-318: Keep RV's Unavailable status arm until D14

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->


## Amendment (2026-10-03)

RFC-032 slice 2 split in two: SL-278 keeps the read surface (`D5` remainder);
`D14` moved to the sibling SL-279. The consequence above now names SL-279 as
owner of the `Unavailable`-arm retirement and the DEC-233 amendment. Where the
choice says "slice 2", read SL-279. Basis: the user's direction in the SL-278
session ("let's go b) to keep this contained").
