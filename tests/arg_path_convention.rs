// SPDX-License-Identifier: GPL-3.0-only
//! SL-236 — the CLI's project-root declaration convention, pinned.
//!
//! Every `-p/--path` argument in the tree binds a field named `path` of type
//! `Option<PathBuf>`, and every one of them means "project root". That uniformity
//! is real but entirely conventional: nothing in the language enforces it, and it
//! was measured by hand (204 declarations across 27 files) rather than guaranteed.
//!
//! It is worth pinning on its own merits — a `-p` bound to `project_root`, or typed
//! `String`, is a CLI-surface inconsistency regardless of who reads it. But the
//! sharper reason is that **any** future work which resolves the project root
//! generically — looking the argument up by clap arg id rather than through each
//! typed variant — silently depends on it. SL-236 explored exactly that (see design
//! §7 A4) and would have rested on this convention with no compiler support: a
//! newly-added `project_root` field would parse and behave correctly while going
//! unnoticed by the resolver.
//!
//! A source scan rather than a behavioural test, because the drift it catches is
//! invisible at runtime — everything compiles and every behavioural test passes.
//!
//! Two halves, and both are needed:
//!   * every `-p` short flag binds a field named `path` — so no project root hides
//!     under another name;
//!   * every clap-facing field named `path` is `Option<PathBuf>` — so a by-id,
//!     typed lookup cannot silently miss one.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::{Path, PathBuf};

mod common;

/// One `#[arg(…)]`-annotated field found in the source tree.
struct ArgField {
    file: String,
    name: String,
    short_p: bool,
    option_pathbuf: bool,
}

fn rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("read src dir").flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// `Option<PathBuf>`, structurally — the exact shape `try_get_one::<PathBuf>`
/// requires. A `String`, a bare `PathBuf`, or an `Option<String>` all fail here.
fn is_option_pathbuf(ty: &syn::Type) -> bool {
    let syn::Type::Path(tp) = ty else {
        return false;
    };
    let Some(seg) = tp.path.segments.last() else {
        return false;
    };
    if seg.ident != "Option" {
        return false;
    }
    let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
        return false;
    };
    let Some(syn::GenericArgument::Type(syn::Type::Path(inner))) = args.args.first() else {
        return false;
    };
    inner
        .path
        .segments
        .last()
        .is_some_and(|s| s.ident == "PathBuf")
}

/// True iff this `#[arg(…)]` attribute declares `short = 'p'`.
fn declares_short_p(attr: &syn::Attribute) -> bool {
    if !attr.path().is_ident("arg") {
        return false;
    }
    let mut found = false;
    // A malformed/unparseable `arg(…)` is not evidence of `short = 'p'`; ignore it
    // rather than fail — the compiler already rejects genuinely broken attributes.
    let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("short")
            && let Ok(value) = meta.value()
            && let Ok(lit) = value.parse::<syn::LitChar>()
            && lit.value() == 'p'
        {
            found = true;
        }
        Ok(())
    });
    found
}

fn collect_fields(file: &Path) -> Vec<ArgField> {
    let text = std::fs::read_to_string(file).expect("read source");
    let label = file
        .strip_prefix(common::repo_root())
        .unwrap_or(file)
        .display()
        .to_string();
    fields_from_source(&text, &label)
}

/// The scan proper, over source TEXT — split out so the predicates can be pinned
/// against fixtures rather than only against a tree that currently satisfies them.
fn fields_from_source(text: &str, label: &str) -> Vec<ArgField> {
    let Ok(ast) = syn::parse_file(text) else {
        return Vec::new();
    };
    let label = label.to_string();

    let mut found = Vec::new();
    let mut visit_fields = |fields: &syn::Fields| {
        for field in fields {
            let Some(name) = field.ident.as_ref() else {
                continue;
            };
            let is_arg = field.attrs.iter().any(|a| a.path().is_ident("arg"));
            let short_p = field.attrs.iter().any(declares_short_p);
            if !is_arg {
                continue;
            }
            found.push(ArgField {
                file: label.clone(),
                name: name.to_string(),
                short_p,
                option_pathbuf: is_option_pathbuf(&field.ty),
            });
        }
    };

    for item in &ast.items {
        match item {
            syn::Item::Struct(s) => visit_fields(&s.fields),
            syn::Item::Enum(e) => {
                for variant in &e.variants {
                    visit_fields(&variant.fields);
                }
            }
            _ => {}
        }
    }
    found
}

fn all_arg_fields() -> Vec<ArgField> {
    let src = common::repo_root().join("src");
    let mut files = Vec::new();
    rust_sources(&src, &mut files);
    assert!(
        files.len() > 50,
        "sanity: expected to scan the whole src tree, found {} files",
        files.len()
    );
    files.iter().flat_map(|f| collect_fields(f)).collect()
}

/// ANTI-VACUITY: the predicates must actually catch the drift they exist to catch.
/// Without this, both assertions below would pass forever on a scan that silently
/// stopped recognising `#[arg(…)]` — a green test proving nothing.
#[test]
fn scan_detects_the_violations_it_polices() {
    const FIXTURE: &str = r#"
        struct Good { #[arg(short = 'p', long)] path: Option<PathBuf> }
        struct LongOnly { #[arg(long)] path: Option<PathBuf> }
        struct WrongName { #[arg(short = 'p', long)] project_root: Option<PathBuf> }
        struct WrongType { #[arg(short = 'p', long)] path: String }
        struct OptionWrongInner { #[arg(long)] path: Option<String> }
        struct NotAnArg { path: Option<PathBuf> }
        enum Verb { New { #[arg(short = 'p', long)] path: Option<PathBuf> } }
    "#;
    let fields = fields_from_source(FIXTURE, "fixture.rs");

    let named: Vec<&str> = fields.iter().map(|f| f.name.as_str()).collect();
    assert!(
        !named.contains(&"path") || fields.len() == 6,
        "the plain (non-`#[arg]`) field must be ignored; saw {named:?}"
    );
    assert_eq!(
        fields.len(),
        6,
        "expected the six `#[arg]` fields (incl. the enum-variant one), saw {named:?}"
    );

    // The `-p`-not-named-`path` rule fires on WrongName and nothing else.
    let wrong_name: Vec<&str> = fields
        .iter()
        .filter(|f| f.short_p && f.name != "path")
        .map(|f| f.name.as_str())
        .collect();
    assert_eq!(
        wrong_name,
        ["project_root"],
        "must catch a renamed `-p` field"
    );

    // The type rule fires on both the bare `String` and the wrong inner type.
    let wrong_type = fields
        .iter()
        .filter(|f| f.name == "path" && !f.option_pathbuf)
        .count();
    assert_eq!(
        wrong_type, 2,
        "must catch `String` AND `Option<String>` bound to `path`"
    );

    // And the enum-variant declaration is seen — most doctrine verbs are newtype
    // variants, so missing these would blind the scan to nearly the whole tree.
    assert!(
        fields.iter().filter(|f| f.short_p).count() == 4,
        "enum-variant `#[arg]` fields must be scanned, not just struct fields"
    );
}

/// Positive control: the scan actually finds the declarations it claims to police.
/// Without this, every assertion below would pass vacuously if the walk broke.
#[test]
fn scan_finds_the_project_root_declarations() {
    let fields = all_arg_fields();
    let short_p = fields.iter().filter(|f| f.short_p).count();
    let named_path = fields.iter().filter(|f| f.name == "path").count();
    assert!(
        short_p > 150,
        "expected the tree's many `-p` declarations; found {short_p}"
    );
    assert!(
        named_path >= short_p,
        "every `-p` is a `path`, so `path` fields ({named_path}) cannot be fewer \
         than `-p` declarations ({short_p})"
    );
}

/// Half one: no project root hides under another field name. A `-p` bound to
/// anything but `path` is invisible to the guard's lookup.
#[test]
fn every_short_p_binds_a_field_named_path() {
    let offenders: Vec<String> = all_arg_fields()
        .into_iter()
        .filter(|f| f.short_p && f.name != "path")
        .map(|f| format!("{}: field `{}`", f.file, f.name))
        .collect();
    assert!(
        offenders.is_empty(),
        "`-p` must always bind a field named `path` — the clap arg id IS the field \
         name, so a differently-named field is invisible to any by-id project-root \
         lookup, and inconsistent on the CLI surface besides:\n  {}",
        offenders.join("\n  ")
    );
}

/// Half two: a typed, by-id lookup cannot silently miss. `try_get_one::<PathBuf>`
/// returns `Err` for any other bound type, which a caller folds to "no explicit
/// root" — indistinguishable from the argument not being passed at all.
#[test]
fn every_clap_path_field_is_option_pathbuf() {
    let offenders: Vec<String> = all_arg_fields()
        .into_iter()
        .filter(|f| f.name == "path" && !f.option_pathbuf)
        .map(|f| format!("{}: field `path`", f.file))
        .collect();
    assert!(
        offenders.is_empty(),
        "a clap `path` argument must be `Option<PathBuf>` — a by-id lookup reads it \
         with `try_get_one::<PathBuf>`, and any other bound type resolves to \
         `None`, indistinguishable from the flag being absent:\n  {}",
        offenders.join("\n  ")
    );
}
