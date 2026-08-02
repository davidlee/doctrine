# Documentation-only Doctrine changes require only corpus validation

When a change touches only authored Doctrine documentation or entity prose/metadata, run doctrine validate. Do not run doctrine check quick, commit, or gate unless code, build configuration, embedded assets, or another executable surface also changed. The code-oriented gates add no relevant evidence for a doc-only delta and may introduce unrelated jail or toolchain failures.

Source: direct maintainer correction on 2026-08-02 after RFC-027 prose-only refinement.