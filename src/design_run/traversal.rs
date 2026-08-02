// SPDX-License-Identifier: GPL-3.0-only
//! Cursor and traversal posture — two types, deliberately (DEC-060, EX-7).
//!
//! *Where* attention sits and *how* the map is being walked are independent
//! facts. Folding them into one value is the accidental hierarchical state
//! machine R3 forbids, and it is also how authority laundering starts: an agent
//! that may propose a posture would inherit permission to move a pinned cursor.
//!
//! Both carry an [`Authority`], so "the user pinned this" is representably
//! distinct from "the agent proposed it" rather than being a convention the
//! caller is trusted to honour.

#![expect(
    dead_code,
    reason = "SL-233: no consumer yet — EX-8 fixes the suite at eight tests, none of which reach this module; PHASE-03/04 are its first callers"
)]

use serde::{Deserialize, Serialize};

use super::ids::DesignId;
use super::refusal::Refusal;

/// Who set a traversal fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Authority {
    /// The agent proposed it while decomposing; freely re-proposable.
    AgentProposed,
    /// The user chose it; an agent proposal may still supersede it.
    UserPinned,
    /// The user fixed it; an agent proposal is refused, not silently applied.
    UserLocked,
}

/// How the map is currently being walked (design §5.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Posture {
    /// Establish major branches across the frontier.
    Breadth,
    /// Pursue one consequential or blocking branch.
    Depth,
}

/// The traversal posture and who chose it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TraversalPosture {
    posture: Posture,
    authority: Authority,
}

impl TraversalPosture {
    /// A posture set by `authority`.
    pub(crate) const fn set(posture: Posture, authority: Authority) -> Self {
        TraversalPosture { posture, authority }
    }

    /// The posture.
    pub(crate) const fn posture(self) -> Posture {
        self.posture
    }

    /// Who set it.
    pub(crate) const fn authority(self) -> Authority {
        self.authority
    }
}

impl Default for TraversalPosture {
    /// Adaptive traversal starts breadth-first, proposed by the agent.
    fn default() -> Self {
        TraversalPosture::set(Posture::Breadth, Authority::AgentProposed)
    }
}

/// Where attention currently sits, and on whose authority.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Cursor {
    at: Option<DesignId>,
    authority: Option<Authority>,
}

impl Cursor {
    /// Place the cursor on `authority`'s say-so. A user may always move it.
    #[must_use]
    pub(crate) fn placed(at: DesignId, authority: Authority) -> Self {
        Cursor {
            at: Some(at),
            authority: Some(authority),
        }
    }

    /// Where the cursor sits.
    pub(crate) const fn at(&self) -> Option<&DesignId> {
        self.at.as_ref()
    }

    /// On whose authority.
    pub(crate) const fn authority(&self) -> Option<Authority> {
        self.authority
    }

    /// An agent proposal to move the cursor.
    ///
    /// Refused against a [`Authority::UserLocked`] cursor. That refusal is the
    /// whole reason authority is a field: without it, "the agent must not move a
    /// locked cursor" is a rule someone has to remember at every call site.
    pub(crate) fn propose(&self, at: DesignId) -> Result<Cursor, Refusal> {
        if self.authority == Some(Authority::UserLocked) {
            return Err(Refusal::CursorLocked {
                at: self.at.clone(),
            });
        }
        Ok(Cursor::placed(at, Authority::AgentProposed))
    }
}
