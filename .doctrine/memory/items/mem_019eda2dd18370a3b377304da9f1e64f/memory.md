# SL-095: old_policy needed for cross-kind record supersession status flips

When merging StorageTarget dispatch with cross-kind record supersession, the TypedArray arm must use old_policy.superseded_status (from OLD's kind) not policy.superseded_status (from NEW's kind) when flipping OLD's status.
