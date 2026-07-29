# Scenario Entity Sketch

## Problem

User stories do not behave like a product altitude.

A story may describe:

* behaviour contained within one feature;
* a journey across several features;
* interaction between several actors;
* a failure or recovery path;
* a temporary unit of delivery planning.

Treating stories as children of features therefore misrepresents many of them. Treating every story as an evergreen entity would also import substantial agile methodology into the normative model.

The useful underlying concept is broader than a user story: a **scenario**.

## Proposed Concept

A scenario is a concrete path through the product or system under stated conditions.

It describes:

* who or what participates;
* the conditions in which the interaction begins;
* the event or goal that drives it;
* the significant interactions;
* the resulting outcome.

```text
Scenario:
  Given these conditions,
  when this event or goal arises,
  these actors interact,
  producing this outcome.
```

Scenarios are orthogonal to both product altitude and C4 level.

```text
Product structure:
  Domain → Capability → Feature → Feature…

Technical structure:
  System Context → Container → Component → Code

Cross-sectional behaviour:
  Scenario
```

## Product Scenarios

A product scenario describes behaviour from the perspective of a user, operator, customer, or other product actor.

```text
Scenario: Recover a compromised account

Actor:
  Account holder

Goal:
  Regain secure control of the account

Preconditions:
  The account holder no longer trusts the active sessions.

Traverses:
  - Identity verification
  - Account recovery
  - Session management
  - Security notifications

Outcome:
  Access is restored and existing sessions are invalidated.
```

A product scenario may traverse several sibling features or even several capabilities. It therefore should not require a structural parent beneath one feature.

## Technical Scenarios

A technical scenario describes a concrete system interaction, operating condition, or failure path.

```text
Scenario: Recovery token is redeemed concurrently

Actors:
  Recovery API
  Token store
  Session service
  Notification worker

Preconditions:
  Two requests present the same valid token concurrently.

Outcome:
  Exactly one redemption succeeds.
  Existing sessions are revoked once.
  The duplicate request receives an idempotent response.
```

Technical scenarios are useful for describing:

* component interactions;
* request and message flows;
* concurrency;
* retries and idempotency;
* dependency failure;
* degraded operation;
* recovery;
* migration;
* compatibility;
* security boundaries;
* performance-sensitive paths.

They occupy similar territory to sequence diagrams, use cases, executable examples, and architecture quality-attribute scenarios.

## Relationship to Requirements

Requirements and scenarios serve different purposes.

```text
Requirement:
  What must be true.

Scenario:
  A concrete path through which relevant behaviour occurs.
```

A scenario may:

* exercise several requirements;
* illustrate why a requirement exists;
* expose interactions between requirements;
* provide context for technical design;
* supply a basis for acceptance or system tests.

A scenario is not automatically normative. Normative obligations remain requirements unless the model explicitly declares scenario outcomes to be binding.

```text
Scenario ── exercises ── Requirement
Scenario ── demonstrates ── Requirement
Scenario ── tested by ── Test or Evidence
```

## Relationship to Specs

Scenarios may be embedded in or referenced by both product and technical specs.

```text
Scenario ── n:n ── Product Spec
Scenario ── n:n ── Technical Spec
```

They may also relate directly to structural entities:

```text
Scenario ── traverses ── Feature
Scenario ── involves ── Container
Scenario ── involves ── Component
```

A scenario embedded in a spec should remain independently addressable when it has graph value beyond that one document.

## Refinement

A high-level product scenario may be refined by several technical scenarios.

```text
Product scenario:
  Customer recovers a compromised account

Technical refinements:
  ├── Successful token redemption
  ├── Expired recovery token
  ├── Concurrent token redemption
  ├── Notification service unavailable
  └── Session revocation partially fails
```

This is refinement rather than altitude. The technical scenarios explain different ways the broader product scenario may be realised or challenged.

## User Stories

User-story syntax may be supported as a compact product-facing representation:

```text
As an account holder,
I want to recover a compromised account,
so that I can regain secure control.
```

This does not require `User Story` to become a distinct entity type.

Possible treatments include:

1. a rendering or authoring format for a product scenario;
2. an optional `story` field on a scenario;
3. a transient delivery artefact linked to an evergreen scenario;
4. an inline shorthand that is promoted to a first-class scenario only when independent identity is useful.

The ontology should model the scenario, not the sentence template.

## Suggested Shape

```yaml
id: SCN-014
title: Recover a compromised account

perspective: product # product | technical | operational

actors:
  - account-holder

goal: Regain secure control of the account

preconditions:
  - the account exists
  - the account holder can complete identity verification

trigger:
  - the account holder begins recovery

outcomes:
  - account access is restored
  - existing sessions are revoked
  - the account holder is notified

traverses:
  - FEAT-021
  - FEAT-034
  - FEAT-041

requirements:
  - REQ-014
  - REQ-027

embedded_in:
  - PRD-008
```

Fields should remain optional enough to support both terse examples and detailed interaction descriptions.

## When a Scenario Should Be First-Class

A scenario warrants independent identity when it needs one or more of:

* reuse across several specs;
* relationships to several features or components;
* refinement into product and technical views;
* traceability to requirements;
* linkage to tests or evidence;
* independent revision history;
* stable reference from reviews and decisions.

A small example used only to clarify one paragraph may remain inline.

## Evergreen Versus Transient Scenarios

Not every backlog story belongs in the normative model.

An evergreen scenario captures a durable and significant example of product or system behaviour.

A transient story may exist only to coordinate a particular delivery increment.

```text
Evergreen:
  A customer recovers a compromised account.

Transient:
  Add the temporary migration path required for the October rollout.
```

The transient item may reference the evergreen scenario, relevant requirements, and affected specs without becoming part of the lasting product ontology.

## Open Questions

1. Whether scenarios are informative by default or may be explicitly normative.
2. Whether product and technical perspectives are facets of one entity or separate scenario kinds.
3. Whether inline scenarios are promoted automatically or only by explicit author action.
4. Whether variants are fields within one scenario or separate scenarios linked by `refines`, `alternative_to`, or `fails_during`.
5. Whether test and evidence relationships belong in the initial implementation or a later extension.

## Provisional Direction

Introduce `Scenario` as a neutral, cross-cutting entity which may embed in product or technical specs.

Treat user stories as one optional notation or delivery-facing projection of product scenarios.

Keep scenarios separate from:

* product altitude;
* C4 level;
* requirements;
* epics and delivery work.
