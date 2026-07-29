# Product Spec Taxonomy and Recursive Features

## Problem

Product specs require a stable indication of altitude, analogous to the level assigned to technical specs using C4.

The initial taxonomy proposed four product altitudes:

```text
Domain
Capability
Feature
User Story
```

`User Story` does not fit this hierarchy. Stories may traverse several features, vary greatly in scope, and describe a path through the product rather than a smaller structural part of it.

Replacing it with `Epic` would introduce a different problem: epics ordinarily represent transient delivery coordination rather than evergreen product truth.

## Decision

Product altitude has three semantic levels:

```text
Domain
└── Capability
    └── Feature
        └── Feature
            └── …
```

Features may recursively contain other features. Recursive feature containment does not introduce additional altitudes: both parent and child remain `Feature`.

Product and technical classification therefore remain intentionally asymmetric:

```text
Product altitude:
  Domain
  Capability
  Feature

C4 level:
  System Context
  Container
  Component
  Code
```

The classifications are approximately comparable forms of zoom, but they are not required to have matching depth or one-to-one equivalence.

## Semantics

### Domain

A broad and durable product problem space or area of responsibility.

Examples:

* Identity and access
* Billing
* Collaboration
* Project governance

### Capability

A durable ability the product provides within a domain.

Examples:

* Account recovery
* Subscription management
* Concurrent editing
* Requirements traceability

A capability describes what the product is able to do without committing to one particular interface or implementation.

### Feature

A coherent, observable part of product behaviour.

Examples:

* Recover an account by email
* Revoke active sessions during recovery
* Preview an upcoming invoice
* Compare conflicting document revisions

Features may be large or small. Their defining property is not size, but that they identify a meaningful part of the product’s behaviour.

## Recursive Feature Containment

A feature may parent another feature when the child is a narrower, independently meaningful part of the parent’s behaviour.

```text
Feature: Account recovery
├── Feature: Recover by email
├── Feature: Recover by passkey
└── Feature: Revoke existing sessions
```

The relationship means:

> The parent feature includes the child feature.

It must not mean merely:

* the child is being delivered as part of the same project;
* the child is technically required to implement the parent;
* the child happens to be scheduled at the same time;
* the child is a task, requirement, or implementation detail.

Feature recursion is therefore product decomposition, not work decomposition.

### Structural invariants

1. Feature containment must be acyclic.
2. Every child feature must describe coherent product behaviour in its own right.
3. A child must narrow or compose the behaviour of its parent.
4. A parent may retain behaviour not fully represented by its children.
5. Technical components, tasks, migrations, and requirements must not be represented as child features.
6. Arbitrary nesting depth may be accepted by the data model but discouraged by presentation and authoring tools.

A single canonical parent is preferable where the product map is intended to form a navigable hierarchy. Cross-cutting behaviour should be represented through explicit relationships, requirements, or scenarios rather than multiple structural parents.

## Inheritance

Feature ancestry supplies **context**, not automatic normative inheritance.

A child feature inherits its enclosing product location:

```text
Domain → Capability → Feature → Feature
```

It does not silently inherit every embedded requirement, constraint, or decision attached to its parent.

Requirements remain independent n:n entities whose applicability is explicit. Tooling may surface requirements attached to ancestor specs as potentially relevant, but ancestry alone must not create unstated obligations.

This distinction avoids conflating:

* product containment;
* requirement applicability;
* delivery dependency;
* implementation structure.

## Requirements

Requirements remain independent entities which may be embedded in or associated with multiple product and technical specs.

```text
Requirement ── n:n ── Product Spec
Requirement ── n:n ── Technical Spec
```

A requirement states an obligation. Its identity and applicability do not depend on the altitude of any one containing spec.

## Epics

`Epic` is not a product altitude.

An epic ordinarily groups work around a delivery objective or planning horizon. It may include:

* several features;
* technical changes;
* migrations;
* research;
* documentation;
* operational preparation.

Its boundaries can change as delivery proceeds and may disappear once the objective is complete.

```text
Evergreen product structure:
  Domain → Capability → Feature → Feature…

Delivery structure:
  Initiative / Epic → Slice / Work Item…
```

An epic may reference product specs and requirements, but it does not form part of the normative product taxonomy.

## Consequences

* Product altitude remains small and semantically stable.
* Feature decomposition can continue as far as the product model requires.
* The taxonomy is not distorted to mirror C4 mechanically.
* Requirements remain orthogonal to product structure.
* Epics remain available for delivery planning without becoming permanent product ontology.
* Cross-feature journeys require a separate representation rather than being forced beneath one feature.
