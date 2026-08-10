## The situation

A payload digest taken over a whole struct —

```rust
let payload = crate::git::sha256(serde_json::to_string(declaration)?.as_bytes());
```

— has a genuinely desirable property: a **new wire field joins the binding
automatically**, rather than by an enumeration somebody must keep current.

The cost is that the test pinning it **cannot stage a natural red**. The moment
the field exists, two payloads differing only in it already digest differently.
A test written afterwards passes on first run, and a test that has never been
seen red is evidence of nothing.

## The control

Perturb the **serde form**, not the test:

```rust
#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]  // real
#[serde(default, skip_serializing)]                            // control
```

`skip_serializing` leaves **deserialisation untouched**, so every other test
that merely *sends* the field stays green — but the field leaves the serialised
form, and therefore leaves the digest material. The digest test must fail.

This is not a proxy for the defect. It is *exactly* the way the property could
stop being true (someone "tidies" the attribute, or a field is added with
`skip_serializing`), and the test is the thing that would notice.

## Read the blast radius too

Count how many tests fail under the control. On SL-249 PHASE-06, exactly one
did — which is itself the finding: no other test depended on that field
reaching the serialised form, so the digest test is the **sole** guard on the
claim. A larger number would have said the serde form was load-bearing in
places worth knowing about. Either answer is information; not looking is the
mistake.

## Pair it with the inspection half

"By construction" has a second half a test cannot see: that **no enumeration was
extended**. Grep the digest expression and assert the call-site count:

```
grep -n "serde_json::to_string(declaration)" src/commands/design.rs
```

Exclude doc comments from the count — a test that documents the expression will
otherwise inflate it and read as a second digest domain.
