# IMP-064: SL-048 OD-3: migrate gov supersedes typed array to [[relation]] LifecycleOnly slot; repoint supersede verb

SL-062 close-time follow-up F3 (design §9). SL-062's `supersede` verb writes the
CURRENT canonical typed `[relationships].supersedes` array (Q3 decision (i)). SL-048
OD-3 migrates gov `supersedes` typed→`[[relation]]` (the reserved `LifecycleOnly`
slot); when it lands, repoint the verb's `supersedes_field`. The transaction shape is
storage-agnostic, so the migration is contained to the field pointer.
