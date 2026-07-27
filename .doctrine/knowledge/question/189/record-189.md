# QUE-189: Default section review posture

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

Which reviewer lanes should a newly started managed design run require by
default?

1. Human section review, preserving today's interactive design contract;
   adversarial section review is first-class but opt-in.
2. Adversarial review followed by human review for every section.
3. Adversarial review as the default proxy, with human section review opt-in.

This default affects cost and interaction cadence, not capability: DEC-073
allows the run policy to be changed during the design.
