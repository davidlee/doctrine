<!-- GENERATED — rendered from the wire-payload contract your installed
     `doctrine` binary parses with, and pinned to it by test. Not hand-editable,
     and not overridable: an edited copy would describe a payload the binary
     does not accept. -->

# Design run — the apply payload contract

Every key a design-run submission may carry, what may be sent under it, whether
it may be omitted and what omission means, and what happens to a key this
contract does not list. Where a variant's payload sits is a function of the
enum's tagging and that variant's payload together, so the rendering states it
per variant rather than per type.

```text
payload ApplyRequest  unknown-keys: silently-dropped   (a misspelt key is discarded, exit 0)
  run_uid            text                      required
  known_revision     integer                   required
  submission_id      text                      required
  adopt_authored     AdoptAuthored             optional
  traversal          TraversalDeclaration      optional
  stage              StageDeclaration          optional
  acceptance         AcceptanceDeclaration     optional
  declare            [Declaration]             optional
  delegation         DelegationAct             optional
  discharge          DischargeDeclaration      optional
  review_policy      ReviewPolicyDeclaration   optional
  checkpoint_act     CheckpointActDeclaration  optional
  agent_declaration  AgentActDeclaration       optional

type AdoptAuthored  unknown-keys: silently-dropped   (a misspelt key is discarded, exit 0)
  fingerprint  text              required
  sections     {id(sec-): text}  optional

type TraversalDeclaration  unknown-keys: silently-dropped   (a misspelt key is discarded, exit 0)
  pin        id(inq-)   sparse   (omit persists · null clears)
  cursor     id(inq-)   sparse   (omit persists · null clears)
  posture    Posture    optional
  authority  Authority  optional

type StageDeclaration  unknown-keys: silently-dropped   (a misspelt key is discarded, exit 0)
  to      Stage  required
  reason  text   optional

type AcceptanceDeclaration  unknown-keys: silently-dropped   (a misspelt key is discarded, exit 0)
  basis  text  required
  turn   text  optional

type Declaration  unknown-keys: refused   (a misspelt key is refused)
  subject     id(inq-|sec-|att-|fnd-|cp-)  required
  question    text                         sparse   (omit persists · null clears)
  needs       [id(inq-)]                   sparse   (omit persists · null clears)
  parent      id(inq-)                     sparse   (omit persists · null clears)
  provenance  Provenance                   optional
  lifecycle   InquiryLifecycle             optional
  body        text                         optional
  attests     id(sec-)                     optional
  reviewer    Reviewer                     optional
  concerns    id(sec-)                     optional
  summary     text                         optional
  blocking    boolean                      optional
  resolution  text                         optional
  disposes    id(inq-)                     optional
  dispose     Dispose                      optional

type CreateRecord  unknown-keys: silently-dropped   (a misspelt key is discarded, exit 0)
  kind        knowledge::RecordKind                     required
  title       text                                      required
  slug        text                                      optional
  body        text                                      optional
  facet       {knowledge::RecordKind chosen by kind: WireFacetValue}  optional
  acceptance  AcceptanceDeclaration                     optional

type DischargeDeclaration  unknown-keys: silently-dropped   (a misspelt key is discarded, exit 0)
  step     text            required
  outcome  DischargeClaim  required
  reason   text            optional

type ReviewPolicyDeclaration  unknown-keys: silently-dropped   (a misspelt key is discarded, exit 0)
  policy      ReviewPolicy           required
  acceptance  AcceptanceDeclaration  required

type CheckpointActDeclaration  unknown-keys: refused   (a misspelt key is refused)
  act          ActKind                required
  acceptance   AcceptanceDeclaration  required
  disposition  ReviewDisposition      optional

type AgentActDeclaration  unknown-keys: refused   (a misspelt key is refused)
  act    AgentAct  required
  basis  text      required
  turn   text      optional

enum Posture  tagging: bare
  breadth
  depth

enum Authority  tagging: bare
  agent-proposed
  user-pinned
  user-locked

enum Stage  tagging: bare
  exploring
  inquiring
  drafting
  reviewing
  locked

enum Provenance  tagging: internal("provenance")   variant keys sit BESIDE "provenance"
  user-directed
  agent-proposed
  shaping-question  record       text      required
  imported-prose    section      id(sec-)  required
                    line         integer   required
                    label        text      required
                    fingerprint  text      required

enum InquiryLifecycle  tagging: bare
  open
  resolved
  deferred
  pruned

enum Reviewer  tagging: bare
  human
  adversarial

enum Dispose  tagging: internal("form")   variant keys sit BESIDE "form"
  create       → CreateRecord's keys, inlined
  adopt        record  text  required
  unresolved   note    text  required
  non-durable  note    text  required

enum WireFacetValue  tagging: untagged   discriminated by JSON shape alone
  ‹no token›  [text]
  ‹no token›  text

enum DelegationAct  tagging: internal("act")   variant keys sit BESIDE "act"
  export   id          id(dlg-)       required
           obligation  id(inq-)       required
  propose  id          id(dlg-)       required
           by          text           required
           summary     text           required
           declare     [Declaration]  optional
  accept   id          id(dlg-)       required
  refuse   id          id(dlg-)       required
           reason      text           required

enum DischargeClaim  tagging: bare
  attested
  skipped

enum ReviewPolicy  tagging: bare
  human-only
  adversarial-only
  human-then-adversarial
  adversarial-then-human

enum ActKind  tagging: bare
  governance-confirmed
  graph-reviewed
  blocking-set-declared
  sufficiency-accepted
  drafting-ready
  section-reviewed
  review-disposed
  design-accepted

enum ReviewDisposition  tagging: external   payload nests UNDER the token
  conducted  { review  text  required }
  waived     { reason  text  required }

enum AgentAct  tagging: external   payload nests UNDER the token
  blocking-set-declared  { blocking  [id(inq-)]  required }
  drafting-ready         — a BARE STRING, not an object

extern knowledge::RecordKind  unknown-keys: refused   (a misspelt key is refused)
  Each token below is one admissible value; the rows under it are the keys it opens.
  assumption  claim            text                                      optional
              confidence       one of: low | medium | high               optional
              basis            one of: observation | prior-art | design-inference | external-source | operator-judgement  optional
              validation_plan  text                                      optional
              validated_by     text                                      optional
              validated_on     text                                      optional
              invalidated_by   text                                      optional
              invalidated_on   text                                      optional
  decision    context          text                                      optional
              choice           text                                      optional
              alternatives     [text]                                    optional
              rationale        text                                      optional
              consequences     [text]                                    optional
              decided_by       text                                      optional
              decided_on       text                                      optional
  question    question         text                                      optional
              why_matters      text                                      optional
              answer           text                                      optional
              answered_by      text                                      optional
              answered_on      text                                      optional
  constraint  statement        text                                      optional
              source           one of: canon | adr | external | technical | legal | compatibility | operator  optional
              applies_to       [text]                                    optional
              waiver_reason    text                                      optional
              waived_by        text                                      optional
              waived_on        text                                      optional
  evidence    datum            text                                      optional
              provenance       one of: inspection | experiment | reproduction | citation  optional
              confidence       one of: low | medium | high               optional
  hypothesis  proposition      text                                      optional
              predicts         text                                      optional
  concept     ‹opens no keys›
```
