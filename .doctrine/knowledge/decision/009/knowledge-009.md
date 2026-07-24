
SL-226 Q3: bare `doctrine graph` (no focus, no filters) emits the whole-corpus
projection silently — no gate, no --all, no stderr size warning. The verb's
contract is "emit the projection you asked for"; downstream gvpr/dot selection
is the composability point. `--depth` accepts any value >= 0, default 1, no
silent clamp (the frontend's [0,3] clamp is a viewport concern, not a stdout
one); depth 0 = focus node alone, matching web BFS semantics.
