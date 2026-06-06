#!/usr/bin/env bash
set -euo pipefail

# Regenerates the <!-- BEGIN:readme-index --> … <!-- END:readme-index -->
# section of README.md from the committed spec corpus.
#
# Source of truth: the authored spec TOML/MD under .doctrine/spec/<subtype>/
# (the committed, reviewed tier of doctrine's storage model — not runtime state).
# Doctrine-flavoured port of the external decision register/scripts/refresh-readme-index.sh.
#
# Subtypes: Product Specifications (PRD-*), Technical Specifications (SPEC-*).

README="README.md"
SPEC_ROOT=".doctrine/spec"

if ! grep -q 'BEGIN:readme-index' "$README"; then
  echo "error: no <!-- BEGIN:readme-index --> marker in $README" >&2
  exit 1
fi

# Pull a scalar string field (title = "…") out of an authored spec/slice TOML.
toml_str() { grep -m1 "^$2 = " "$1" | sed "s/^$2 = \"\(.*\)\"/\1/"; }

# Phase rollups (N/M, —, !N, ?N) are CLI-derived from the gitignored state tree
# — `slice list` is the only source. Map id -> rollup token; — when unavailable.
declare -A ROLLUP
if command -v doctrine >/dev/null 2>&1; then
  while read -r _id _roll; do ROLLUP["$_id"]="$_roll"; done < <(
    doctrine slice list 2>/dev/null | awk 'NR>1 {
      roll="—"
      for (i=2;i<=NF;i++) if ($i ~ /^(—|[0-9]+\/[0-9]+|[!?][0-9]+)$/) { roll=$i; break }
      print $1, roll
    }' || true
  )
fi

section=""

# Append one spec-subtype block. $1 = subtype dir, $2 = id prefix, $3 = heading.
# Iterates the `NNN-slug` symlinks so id, slug, and link path come from one name.
add_subtype() {
  local subdir="$SPEC_ROOT/$1" prefix="$2" heading="$3"
  local sym base id md toml title status block=""
  [[ -d "$subdir" ]] || return 0
  for sym in $(find "$subdir" -maxdepth 1 -type l -name '[0-9]*' | sort); do
    base="$(basename "$sym")"
    id="${base%%-*}"
    md="$sym/spec-$id.md"
    toml="$sym/spec-$id.toml"
    [[ -f "$md" && -f "$toml" ]] || continue
    title="$(toml_str "$toml" title)"
    status="$(toml_str "$toml" status)"
    block+="- [$prefix-$id — $title]($md) — \`$status\`\n"
  done
  if [[ -n "$block" ]]; then
    section+="\n### $heading\n\n$block"
  fi
}

add_subtype product PRD "Product Specifications"
add_subtype tech SPEC "Technical Specifications"

# Compact slice index: title -> design, "scope" -> scope doc, (rollup) from CLI.
add_slices() {
  local sym base id title rollup design block=""
  for sym in $(find ".doctrine/slice" -maxdepth 1 -type l -name '[0-9]*' | sort); do
    base="$(basename "$sym")"
    id="${base%%-*}"
    [[ -f "$sym/slice-$id.toml" ]] || continue
    title="$(toml_str "$sym/slice-$id.toml" title)"
    rollup="${ROLLUP[$id]:-—}"
    if [[ -f "$sym/design.md" ]]; then
      design="[$title]($sym/design.md)"
    else
      design="$title"
    fi
    block+="- $design | [scope]($sym/slice-$id.md) ($rollup)\n"
  done
  if [[ -n "$block" ]]; then
    section+="\n### Slices\n\n$block"
  fi
}

add_slices

# Drop the leading literal "\n" so the first heading abuts the BEGIN marker.
section="${section#\\n}"

# Replace the marked section. awk -v expands the literal \n escapes.
tmpfile=$(mktemp)
awk -v section="$section" '
  /<!-- BEGIN:readme-index -->/ { print; printf "%s", section; skip=1; next }
  /<!-- END:readme-index -->/   { skip=0 }
  skip { next }
  { print }
' "$README" > "$tmpfile"

mv "$tmpfile" "$README"
echo "readme index refreshed"
