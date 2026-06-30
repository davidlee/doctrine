# Promoting a local memory to a shipped master requires a body scrub, not just a TOML re-class

Re-classing toml signature (repo='', anchor=none, scope floor) is necessary but NOT sufficient: the body must also be scrubbed of host-local refs (local memory uids, ISS-/SL-/REC-/REQ- ids) or it ships host-project state, violating POL-002
