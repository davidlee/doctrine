A question re-word on an inquiry node emits `node_question_changed`, naming the node as its subject and carrying no payload terms. It is emitted only when the declared text differs from the held text; a same-text re-declaration and a node's creation (already reported by `node_created`) emit none.

Why term-free: `DEC-237` holds that subject id and event kind answer *did what I sent land?* completely. The engine computes no digests (section digests arrive precomputed from the shell), so an old/new digest pair would need new shell plumbing for a value no reader can interpret; a prose term would risk becoming the widest payload and moving the change log's projection-bounds exemplar. The text itself is one `design show` away.

Why only on change: `REQ-478` obliges a row per recorded mutation, and re-sending held text mutates nothing.

Supersedes the `SL-233`-era code comment that a question change is "state, not delta"; `REQ-478` postdates it.