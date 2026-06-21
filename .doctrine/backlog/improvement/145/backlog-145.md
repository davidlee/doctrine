# IMP-145: Entity info command for metadata and file path surfaces

SL-139 is deliberately putting `show --filepaths` on the existing show surface as a narrow compatibility/readability improvement, but the better long-term home for summary metadata and file location details is likely a separate `info`-style read verb.

Explore and design a uniform entity `info` command (name TBD) that can show concise metadata, storage/file paths, and potentially other non-body summary details without overloading `show`, whose primary job is readable entity body/reconstruction. Reconsider whether filepath modes should live there, and whether both augmented and paths-only modes are still useful once the info surface exists.

Keep this separate from SL-139 unless its design discovers a hard dependency.
