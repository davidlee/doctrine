// SPDX-License-Identifier: GPL-3.0-only
//! Run-local identity and content fingerprints.
//!
//! [`DesignId`] holds a private field and is reachable only through
//! [`DesignId::parse`] — **one validating constructor, and no other way to make
//! one**. That structure is the point: the [`DESIGN_ID_BYTES`] admission bound
//! lives *inside* this constructor and every construction site inherits it,
//! rather than each path having to remember (projection-bounds sketch
//! § *Enforcement — structural, not textual*, item 2). With ids now rendered
//! whole, a single admission path accepting a 33-byte id would break the 32-byte
//! row premise and the envelope arithmetic with it while every other test still
//! passed.
//!
//! Exceeding the bound is a **refusal, never a trim**: a truncated identity is a
//! *wrong* identity rather than a shorter one, and two distinct subjects that
//! render identically is the failure the layer rule exists to prevent.

use std::fmt;

use serde::{Deserialize, Serialize};

use super::bounds::DESIGN_ID_BYTES;
use super::refusal::Refusal;

/// What a run-local id names. The prefix is part of the id's spelling, so the
/// kind is *derived* from the text rather than supplied alongside it — one
/// source of truth, and a mistyped prefix is a refusal instead of a mismatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum IdKind {
    /// An inquiry-map node (`inq-`).
    Inquiry,
    /// A draft section (`sec-`).
    Section,
    /// A checkpoint disposing an inquiry (`cp-`).
    Checkpoint,
    /// A content-bound review attestation (`att-`).
    Attestation,
    /// A runtime review finding (`fnd-`).
    Finding,
    /// One exported delegation assignment (`dlg-`).
    Delegation,
}

impl IdKind {
    /// Every kind — the closed vocabulary, single-sourced (STD-001).
    pub(crate) const ALL: [IdKind; 6] = [
        IdKind::Inquiry,
        IdKind::Section,
        IdKind::Checkpoint,
        IdKind::Attestation,
        IdKind::Finding,
        IdKind::Delegation,
    ];

    /// The id prefix, including its trailing hyphen.
    pub(crate) const fn prefix(self) -> &'static str {
        match self {
            IdKind::Inquiry => "inq-",
            IdKind::Section => "sec-",
            IdKind::Checkpoint => "cp-",
            IdKind::Attestation => "att-",
            IdKind::Finding => "fnd-",
            IdKind::Delegation => "dlg-",
        }
    }
}

/// May `byte` appear in an id's body?
///
/// `[A-Za-z0-9_-]` is a **choice**, not a derivation, and is recorded as one:
/// `.`, `:`, `/`, `+`, `=` and `@` all satisfy the three constraints the marker
/// grammar actually imposes, so this is emphatically not the maximal safe set.
/// What justifies it is that it is the identifier charset the corpus already
/// uses for entity ids and slugs, and that above the safety floor the cheap
/// direction is conservative: widening admission later is free, narrowing it
/// after ids exist in committed authored documents is a migration.
const fn is_id_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'
}

/// A validated run-local id.
///
/// The field is private and there is no raw constructor: [`DesignId::parse`] is
/// the only route in, so no admission path can bypass validation by building the
/// value directly.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub(crate) struct DesignId {
    raw: String,
}

impl DesignId {
    /// The one validating constructor. Refuses an unknown prefix, an empty body
    /// after the prefix, a body carrying a byte outside [`is_id_byte`], and
    /// anything over [`DESIGN_ID_BYTES`].
    ///
    /// **Total by construction, and the arms are ordered.** Length, then prefix,
    /// then a non-empty body, then the charset — each decidable on any `&str`, so
    /// exactly one of [`Refusal::IdTooLong`], [`Refusal::MalformedId`] or
    /// acceptance results. Whitespace, `>`, `\n`, `\r` and every non-ASCII byte
    /// land in the charset arm, and a bad byte is a *malformed* id rather than an
    /// over-long one.
    ///
    /// The charset arm is what makes the marker grammar's token split
    /// unambiguous (`sketches/marker-grammar.md` answer (a)): an id may not carry
    /// whitespace, or the third token cannot be compared whole; may not carry
    /// `>`, or it could terminate its own comment early; and may not be non-ASCII,
    /// so a byte count equals a column count. It is applied to **every kind**,
    /// not only sections, because every kind's id is rendered into a change row
    /// whose encoding is equally corrupted by a control byte — and two id rules
    /// kept in agreement is one more than the model needs.
    pub(crate) fn parse(raw: &str) -> Result<DesignId, Refusal> {
        if raw.len() > DESIGN_ID_BYTES {
            return Err(Refusal::IdTooLong {
                raw: raw.to_owned(),
                limit: DESIGN_ID_BYTES,
            });
        }
        let body = IdKind::ALL
            .iter()
            .find_map(|kind| raw.strip_prefix(kind.prefix()));
        match body {
            Some(body) if !body.is_empty() && body.bytes().all(is_id_byte) => Ok(DesignId {
                raw: raw.to_owned(),
            }),
            _ => Err(Refusal::MalformedId {
                raw: raw.to_owned(),
            }),
        }
    }

    /// What this id names, derived from its prefix.
    pub(crate) fn kind(&self) -> IdKind {
        IdKind::ALL
            .into_iter()
            .find(|kind| self.raw.starts_with(kind.prefix()))
            .unwrap_or(IdKind::Inquiry)
    }

    /// The id as written.
    pub(crate) fn as_str(&self) -> &str {
        &self.raw
    }
}

impl fmt::Display for DesignId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.raw)
    }
}

impl TryFrom<String> for DesignId {
    type Error = Refusal;

    fn try_from(raw: String) -> Result<DesignId, Refusal> {
        DesignId::parse(&raw)
    }
}

impl From<DesignId> for String {
    fn from(id: DesignId) -> String {
        id.raw
    }
}

/// A content fingerprint. Evidence is bound to one (DEC-066); when the subject's
/// fingerprint moves, evidence recorded against the old one is no longer live.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(crate) struct Fingerprint(String);

impl Fingerprint {
    /// Wrap a digest computed by the shell. The pure layer never hashes — it is
    /// handed the digest as a derived fact.
    pub(crate) fn new(digest: impl Into<String>) -> Fingerprint {
        Fingerprint(digest.into())
    }

    /// The digest as written.
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}
