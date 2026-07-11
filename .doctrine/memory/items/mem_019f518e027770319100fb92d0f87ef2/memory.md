# Dead-symbol claims need full-file symbol grep, not definition-region reads

Before claiming an enum variant/symbol is dead, grep the symbol across EVERY file grep -l flagged in full — a definition-region read plus targeted greps of OTHER files misses a producer later in the same file
