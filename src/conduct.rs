// SPDX-License-Identifier: GPL-3.0-only
//! `conduct` — the orthogonal conduct axis (SL-028 design §5.2, ADR-009 §2).
//!
//! *What* a change is (the slice lifecycle FSM, axis A) is separate from *how it
//! is conducted* (axis B): `actor × autonomy`, declared per lifecycle state. This
//! module owns the pure domain model of axis B — the two enums, the resolved
//! [`Conduct`] pair, the [`ConductConfig`] parsed from a project `doctrine.toml`
//! `[conduct]` table, and the pure [`resolve`] that folds a config + a queried
//! state into the effective posture with baked defaults.
//!
//! **Pure engine tier (ADR-001).** No clock / disk / rng / git here: the
//! `doctrine.toml` *read* lives in the `slice` command shell; [`parse`] and
//! [`resolve`] take owned/borrowed data only.
//!
//! **Advisory, never enforced (F15).** The posture is *declared config* — what the
//! project intends — never a runtime actor attribution or an enforcement decision.
//! `slice status` / `slice show` print it; nothing here gates a write.
//!
//! **Autonomy is exit semantics (F19).** A state's `autonomy` governs advancing
//! *out* of it, so `slice status` resolves the SOURCE state (`resolve(from)`):
//! `reconcile = gate` gates `reconcile → done` (the closure gate) and
//! `plan = gate` gates `plan → ready` (the approved-plan gate).

use std::collections::BTreeMap;

use serde::Deserialize;

/// Axis-B *actor*: who the project declares conducts a state. Advisory config,
/// not a runtime identity (F15). `Author` renames to `"self"` in TOML — `self` is
/// a Rust keyword, so the variant is named `Author` and serde-renamed (F5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Actor {
    Agent,
    #[serde(rename = "self")]
    Author,
    Peer,
    Team,
}

impl Actor {
    /// The short label rendered in the posture string (matches the serde spelling).
    const fn as_str(self) -> &'static str {
        match self {
            Actor::Agent => "agent",
            Actor::Author => "self",
            Actor::Peer => "peer",
            Actor::Team => "team",
        }
    }
}

/// Axis-B *autonomy*: how much latitude exiting a state carries. `auto` advances
/// freely, `draft` proposes for review, `gate` expects an explicit human handoff
/// (the `plan`/`reconcile` defaults). Exit semantics (F19).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Autonomy {
    Auto,
    Draft,
    Gate,
}

impl Autonomy {
    /// The short label rendered in the posture string (matches the serde spelling).
    const fn as_str(self) -> &'static str {
        match self {
            Autonomy::Auto => "auto",
            Autonomy::Draft => "draft",
            Autonomy::Gate => "gate",
        }
    }
}

/// The effective conduct posture for one state — the resolved `actor × autonomy`.
/// Produced by [`resolve`]; rendered by [`Conduct::label`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Conduct {
    pub(crate) actor: Actor,
    pub(crate) autonomy: Autonomy,
}

impl Conduct {
    /// The single posture format shared by both surfaces (DRY — design §5.2,
    /// example `reconcile → done [self/gate]`): `actor-short/autonomy-short`.
    pub(crate) fn label(self) -> String {
        [self.actor.as_str(), "/", self.autonomy.as_str()].concat()
    }
}

/// A single `[conduct.<state>]` override subtable — each field optional, so an
/// override may set just `autonomy` (the documented `plan`/`reconcile` shape) and
/// inherit the `actor` default. Unknown keys are ignored (tolerant parse, F9).
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
struct StateOverride {
    actor: Option<Actor>,
    autonomy: Option<Autonomy>,
}

/// The parsed `[conduct]` table: the two project-wide defaults plus per-state
/// overrides keyed by the lifecycle-state string (`BTreeMap` — `HashMap` is banned by
/// repo clippy). All fields default, so an absent file / absent `[conduct]` key /
/// partial table all yield the empty config, which [`resolve`] reads as the baked
/// defaults (F9 tolerance). `#[serde(default)]` + `flatten` lets the per-state
/// subtables (`[conduct.plan]`, …) collect into `states` while the two scalar
/// `default-*` keys parse alongside them.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub(crate) struct ConductConfig {
    default_actor: Option<Actor>,
    default_autonomy: Option<Autonomy>,
    #[serde(flatten)]
    states: BTreeMap<String, StateOverride>,
}

/// The baked actor default when neither the file nor a state override speaks:
/// `self` everywhere (F9 / design §5.3). Invoker-blind config, not attribution.
const DEFAULT_ACTOR: Actor = Actor::Author;

/// Whether a lifecycle state defaults to `gate` autonomy in a zero-config repo:
/// the two load-bearing human gates — `plan` (the "no code without an approved
/// plan" gate) and `reconcile` (the ADR-003 §8 closure gate). Every other state
/// defaults to `auto` (design §5.3).
fn baked_autonomy(state: &str) -> Autonomy {
    match state {
        "plan" | "reconcile" => Autonomy::Gate,
        _ => Autonomy::Auto,
    }
}

/// Parse a project `doctrine.toml`'s `[conduct]` table from owned text. Pure —
/// the file *read* is the shell's job (the absent-file case never reaches here;
/// the shell passes the default config). Tolerant (F9): a malformed `[conduct]`
/// surfaces as an error to the caller, but absent / partial / unknown-key tables
/// all parse to the defaults. Unknown top-level keys and unknown-state subtables
/// are accepted and ignored by [`resolve`].
pub(crate) fn parse(text: &str) -> anyhow::Result<ConductConfig> {
    Ok(crate::dtoml::parse(text)?.conduct)
}

/// Resolve the effective [`Conduct`] for one lifecycle `state`. Pure, total over
/// ANY string (incl. an out-of-vocab / drifted state — never panics): a per-state
/// override beats the project default beats the baked default, per field
/// independently. `resolve` is keyed by the *queried* state, so unknown-state
/// override subtables are simply never consulted (F9).
pub(crate) fn resolve(cfg: &ConductConfig, state: &str) -> Conduct {
    let over = cfg.states.get(state);
    let actor = over
        .and_then(|o| o.actor)
        .or(cfg.default_actor)
        .unwrap_or(DEFAULT_ACTOR);
    let autonomy = over
        .and_then(|o| o.autonomy)
        .or(cfg.default_autonomy)
        .unwrap_or_else(|| baked_autonomy(state));
    Conduct { actor, autonomy }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- T1: serde of the two enums (the keyword-rename + kebab spellings) ---

    /// Wrapper so we can parse a bare enum value from a TOML scalar.
    #[derive(Deserialize)]
    struct ActorWrap {
        a: Actor,
    }
    #[derive(Deserialize)]
    struct AutonomyWrap {
        a: Autonomy,
    }

    fn parse_actor(s: &str) -> Actor {
        let w: ActorWrap = toml::from_str(&format!("a = \"{s}\"")).expect("actor parse");
        w.a
    }
    fn parse_autonomy(s: &str) -> Autonomy {
        let w: AutonomyWrap = toml::from_str(&format!("a = \"{s}\"")).expect("autonomy parse");
        w.a
    }

    #[test]
    fn actor_author_renames_to_self() {
        assert_eq!(parse_actor("self"), Actor::Author);
        assert_eq!(Actor::Author.as_str(), "self");
    }

    #[test]
    fn actor_other_variants_are_kebab() {
        assert_eq!(parse_actor("agent"), Actor::Agent);
        assert_eq!(parse_actor("peer"), Actor::Peer);
        assert_eq!(parse_actor("team"), Actor::Team);
    }

    #[test]
    fn autonomy_variants_are_kebab() {
        assert_eq!(parse_autonomy("auto"), Autonomy::Auto);
        assert_eq!(parse_autonomy("draft"), Autonomy::Draft);
        assert_eq!(parse_autonomy("gate"), Autonomy::Gate);
    }

    // --- T2: tolerant parse ---

    #[test]
    fn absent_conduct_key_parses_to_defaults() {
        let cfg = parse("title = \"some other doctrine.toml content\"\n").expect("parse");
        assert_eq!(cfg, ConductConfig::default());
    }

    #[test]
    fn empty_text_parses_to_defaults() {
        assert_eq!(parse("").expect("parse"), ConductConfig::default());
    }

    #[test]
    fn full_conduct_table_round_trips() {
        let cfg = parse(
            "[conduct]\n\
             default-actor = \"agent\"\n\
             default-autonomy = \"draft\"\n\
             [conduct.plan]\n\
             autonomy = \"gate\"\n\
             [conduct.reconcile]\n\
             actor = \"team\"\n\
             autonomy = \"gate\"\n",
        )
        .expect("parse");
        assert_eq!(cfg.default_actor, Some(Actor::Agent));
        assert_eq!(cfg.default_autonomy, Some(Autonomy::Draft));
        assert_eq!(
            cfg.states.get("plan").and_then(|o| o.autonomy),
            Some(Autonomy::Gate)
        );
        assert_eq!(
            cfg.states.get("reconcile").and_then(|o| o.actor),
            Some(Actor::Team)
        );
    }

    #[test]
    fn unknown_state_subtable_is_tolerated_not_errored() {
        // F9: a `[conduct.foo]` for a non-vocab state parses without error; it is
        // simply never consulted by `resolve` (keyed by the queried state).
        let cfg = parse("[conduct.foo]\nautonomy = \"gate\"\n").expect("parse");
        assert!(cfg.states.contains_key("foo"));
    }

    /// Canary pinning the accepted TOML shape (toml-error-classification-fragile):
    /// the documented `default-*` kebab keys + a partial subtable that sets only
    /// `autonomy` must keep parsing across toml-crate versions.
    #[test]
    fn canary_documented_shape_parses() {
        let cfg = parse(
            "[conduct]\n\
             default-actor = \"self\"\n\
             default-autonomy = \"auto\"\n\
             [conduct.plan]\n\
             autonomy = \"gate\"\n\
             [conduct.reconcile]\n\
             autonomy = \"gate\"\n",
        )
        .expect("parse");
        // `plan` sets only autonomy → actor inherits the default on resolve.
        assert_eq!(
            resolve(&cfg, "plan"),
            Conduct {
                actor: Actor::Author,
                autonomy: Autonomy::Gate
            }
        );
    }

    // --- T3: resolve with baked defaults + precedence ---

    #[test]
    fn default_state_falls_back_to_self_auto() {
        let c = resolve(&ConductConfig::default(), "started");
        assert_eq!(
            c,
            Conduct {
                actor: Actor::Author,
                autonomy: Autonomy::Auto
            }
        );
    }

    #[test]
    fn plan_and_reconcile_gate_by_default() {
        let cfg = ConductConfig::default();
        assert_eq!(resolve(&cfg, "plan").autonomy, Autonomy::Gate);
        assert_eq!(resolve(&cfg, "reconcile").autonomy, Autonomy::Gate);
        // actor still the baked self default.
        assert_eq!(resolve(&cfg, "plan").actor, Actor::Author);
    }

    #[test]
    fn override_beats_default_per_field() {
        // A `[conduct.ready]` override sets autonomy=gate; actor inherits the
        // project default-actor (= agent). VT-2 precedence.
        let cfg = parse(
            "[conduct]\n\
             default-actor = \"agent\"\n\
             [conduct.ready]\n\
             autonomy = \"gate\"\n",
        )
        .expect("parse");
        assert_eq!(
            resolve(&cfg, "ready"),
            Conduct {
                actor: Actor::Agent,
                autonomy: Autonomy::Gate
            }
        );
    }

    #[test]
    fn override_beats_baked_gate_default() {
        // An explicit `plan` override to `auto` beats the baked gate default.
        let cfg = parse("[conduct.plan]\nautonomy = \"auto\"\n").expect("parse");
        assert_eq!(resolve(&cfg, "plan").autonomy, Autonomy::Auto);
    }

    #[test]
    fn resolve_is_total_over_drifted_state() {
        // An out-of-vocab `from` returns the global default, never panics.
        let c = resolve(&ConductConfig::default(), "totally-made-up-state");
        assert_eq!(
            c,
            Conduct {
                actor: Actor::Author,
                autonomy: Autonomy::Auto
            }
        );
    }

    // --- T7: the shipped reference template ---

    #[test]
    fn shipped_template_is_valid_and_its_defaults_round_trip() {
        // The embedded `doctrine.toml.example` ships fully commented, so parsing
        // it raw yields the baked defaults (it is opt-in, never auto-active).
        let raw = crate::install::asset_text("doctrine.toml.example").expect("embedded template");
        let cfg = parse(&raw).expect("template parses");
        assert_eq!(cfg, ConductConfig::default());

        // Its documented (uncommented) [conduct] block — what a user copies — must
        // round-trip to the baked posture: plan & reconcile gate, all else self/auto.
        let live = raw
            .replace("# [conduct", "[conduct")
            .replace("\n# autonomy", "\nautonomy");
        // Strip the leading-comment scalars too (default-actor / default-autonomy).
        let live = live
            .replace("# default-actor", "default-actor")
            .replace("# default-autonomy", "default-autonomy");
        let cfg = parse(&live).expect("uncommented template parses");
        assert_eq!(resolve(&cfg, "plan").label(), "self/gate");
        assert_eq!(resolve(&cfg, "reconcile").label(), "self/gate");
        assert_eq!(resolve(&cfg, "started").label(), "self/auto");
    }

    // --- T4: label format ---

    #[test]
    fn label_renders_actor_slash_autonomy() {
        assert_eq!(
            Conduct {
                actor: Actor::Author,
                autonomy: Autonomy::Gate
            }
            .label(),
            "self/gate"
        );
        assert_eq!(
            Conduct {
                actor: Actor::Agent,
                autonomy: Autonomy::Auto
            }
            .label(),
            "agent/auto"
        );
        assert_eq!(
            Conduct {
                actor: Actor::Peer,
                autonomy: Autonomy::Draft
            }
            .label(),
            "peer/draft"
        );
    }
}
