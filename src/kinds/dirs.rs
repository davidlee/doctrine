// SPDX-License-Identifier: GPL-3.0-only
//! Entity-tree roots — the one place each kind's authored directory is spelled
//! (STD-001).
//!
//! **This file imports nothing, on purpose.** It is `#[path]`-included by
//! `tests/common/mod.rs` (the CHR-014 idiom this crate already uses for
//! `src/test_support.rs`), so a black-box integration test fixture that plants a
//! `.doctrine/…` tree reads the same bytes the binary compiles instead of
//! hand-typing the path beside it (SL-233 PHASE-04, RV-321 F-4). The rest of
//! `kinds` cannot be included that way — `kinds::resolve` reaches for
//! `crate::fsutil` — which is why the roots live here rather than in `mod.rs`.
//!
//! Visibility is preserved by the re-export list in [`super`]: the roots an
//! origin module needs are `pub(crate)` there, and the rest stay leaf-private.

pub(crate) const SLICE_DIR: &str = ".doctrine/slice";
pub(crate) const CONCEPT_MAP_DIR: &str = ".doctrine/concept-map";
pub(crate) const REV_DIR: &str = ".doctrine/revision";
pub(crate) const REC_DIR: &str = ".doctrine/rec";
pub(crate) const REVIEW_DIR: &str = ".doctrine/review";
pub(crate) const REQUIREMENT_DIR: &str = ".doctrine/requirement";
pub(crate) const RFC_DIR: &str = ".doctrine/rfc";
pub(crate) const ADR_DIR: &str = ".doctrine/adr";
pub(crate) const POLICY_DIR: &str = ".doctrine/policy";
pub(crate) const STANDARD_DIR: &str = ".doctrine/standard";
pub(crate) const PRODUCT_SPEC_DIR: &str = ".doctrine/spec/product";
pub(crate) const TECH_SPEC_DIR: &str = ".doctrine/spec/tech";
pub(crate) const ASSUMPTION_DIR: &str = ".doctrine/knowledge/assumption";
pub(crate) const DECISION_DIR: &str = ".doctrine/knowledge/decision";
pub(crate) const QUESTION_DIR: &str = ".doctrine/knowledge/question";
pub(crate) const CONSTRAINT_DIR: &str = ".doctrine/knowledge/constraint";
pub(crate) const EVIDENCE_DIR: &str = ".doctrine/knowledge/evidence";
pub(crate) const HYPOTHESIS_DIR: &str = ".doctrine/knowledge/hypothesis";
pub(crate) const CONCEPT_DIR: &str = ".doctrine/knowledge/concept";
pub(crate) const ISSUE_DIR: &str = ".doctrine/backlog/issue";
pub(crate) const IMPROVEMENT_DIR: &str = ".doctrine/backlog/improvement";
pub(crate) const CHORE_DIR: &str = ".doctrine/backlog/chore";
pub(crate) const RISK_DIR: &str = ".doctrine/backlog/risk";
pub(crate) const IDEA_DIR: &str = ".doctrine/backlog/idea";
