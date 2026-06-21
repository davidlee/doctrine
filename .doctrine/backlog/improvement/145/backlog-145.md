# IMP-145: Entity info command for metadata and file path surfaces

SL-139 moved the immediate file-location need to a dedicated `paths` verb, leaving `show` as the readable body/reconstruction surface.

Explore and design a uniform entity `info` command (name TBD) for concise non-body summary metadata that does not fit `show` or the narrow `paths` command. Candidate content includes identity, status, lifecycle/posture, relationship counts, tags/facets summaries, and storage hints where useful. Do not duplicate `paths` unless a later design proves summary + paths composition belongs on `info`.

Keep this separate from SL-139 unless its design discovers a hard dependency.
