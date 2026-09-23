When a slice ships prose (a convention, a skill clause) that its design drafted
and the plan transcribes *verbatim*, every phase check passes on a defective
draft: keyword VTs find the text, the render check shows it delivered, and a
single-owner sweep proves it stated once. None of them asks whether it is *right*.

SL-260 hit this twice. DEC-277: two required clauses were absent from the drafts,
caught by the single-owner sweep only because they had zero owners. RV-373 F-1: a
clause present once but over-scoped ("routed finding" where DEC-263 says
demonstrate/probe/control only), caught only by reading shipped text against the
accepted DEC that governs it.

How to apply: at audit (or at phase end for a prose-delivery phase), for each
shipped normative clause, open the DEC it implements and check its qualifiers
(which cases, which routes, which roles) survived into the text. Where the locked
design and an accepted DEC disagree, that is drift between two binding sources —
surface it to the user, don't pick.
