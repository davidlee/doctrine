#!/usr/bin/env nu
# SL-273 PHASE-04 per-occurrence sweep verifier (RV-408 F-1; design 6.3 steps 2-4).
#
# Proves that the shipped-text delta between --base and the post-state is
# exactly the adjudicated inventory rows in scope: every changed word maps to
# one applied row, and every other row is unchanged. The comparison is over
# whitespace-separated words, so rewrap and re-indent are tolerated and wording
# is not. inventory.toml is the single source of truth at run time (notes.md
# "Outcome rule"); a seam repair is recorded there as an amend before it can
# pass.
#
# Usage:
#   nu verify-sweep.nu --base <rev> (--through <U2|U3a|U3b|U3c|U4> | --applied <id,id,...>) [--tree <overlay-dir>]
#
# Exit: 0 = no violations; 1 = violations; 2 = hard error. Anything the
# verifier cannot read or locate is a violation or a hard error, never a skip.

const SLICE_DIR = path self .
const ROOTS = [plugins/doctrine/skills install src]
const UNITS = [U2 U3a U3b U3c U4]
const CHANGING_RECOMMEND = [convert cut-to-help cut-to-lib]
const LOG_ENTITY = "IMP-500"
const CTX = 8
const WIDTH = 80

def die [msg: string] {
  print -e $"HARD ERROR: ($msg)"
  exit 2
}

# notes.md Outcome rule.
def changes [r: record] {
  $r.verdict == "amend" or ($r.verdict == "accept" and $r.recommend in $CHANGING_RECOMMEND)
}

def new-text [r: record] {
  if $r.verdict == "amend" { $r.resolved } else { $r.target }
}

def norm [s: string] {
  $s | str replace -ar '\s+' ' ' | str trim
}

# Base location: the excerpt occurs exactly once in the file, the span exactly
# once inside the excerpt. Byte offsets, consistent with nu's str builtins.
def locate [text: string, r: record] {
  if ($r.excerpt | is-empty) or ($r.span | is-empty) {
    return {ok: false, why: "empty excerpt or span", ea: -1, eb: -1, a: -1, b: -1}
  }
  let parts = ($text | split row $r.excerpt)
  let n = ($parts | length) - 1
  if $n != 1 {
    return {ok: false, why: $"excerpt occurs ($n)x in base", ea: -1, eb: -1, a: -1, b: -1}
  }
  let sp = ($r.excerpt | split row $r.span)
  let m = ($sp | length) - 1
  if $m != 1 {
    return {ok: false, why: $"span occurs ($m)x in excerpt", ea: -1, eb: -1, a: -1, b: -1}
  }
  let ea = ($parts | first | str length)
  let a = $ea + ($sp | first | str length)
  {ok: true, why: "", ea: $ea, eb: ($ea + ($r.excerpt | str length)), a: $a, b: ($a + ($r.span | str length))}
}

# y's span range contains x's (same file, different row).
def contains [y: record, x: record] {
  $y.id != $x.id and $y.file == $x.file and $y.ok and $x.ok and $y.a <= $x.a and $x.b <= $y.b
}

# Expected text of [lo, hi) as segments {text, tags}: applied ranges replaced by
# their new text (tagged with the row), other text tagged with every tag span
# covering it ("" = free). Null when an applied range straddles the window.
def segments [text: string, lo: int, hi: int, arows: list, tspans: list] {
  let ina = ($arows | where {|x| $x.a < $hi and $x.b > $lo })
  if ($ina | any {|x| $x.a < $lo or $x.b > $hi }) { return null }
  let pts = (
    [$lo $hi]
    | append ($ina | each {|x| [$x.a $x.b] } | flatten)
    | append ($tspans | each {|x| [$x.a $x.b] } | flatten)
    | where {|p| $p >= $lo and $p <= $hi }
    | uniq | sort
  )
  if ($pts | length) < 2 { return [] }
  $pts | window 2 | each {|w|
    let p = $w.0
    let q = $w.1
    let hit = ($ina | where {|x| $x.a <= $p and $q <= $x.b })
    if ($hit | is-not-empty) {
      let x = ($hit | first)
      if $p == $x.a { {text: $x.new, tags: $x.id} } else { null }
    } else {
      let t = ($tspans | where {|s| $s.a <= $p and $q <= $s.b } | each {|s| $s.id } | str join ",")
      {text: ($text | str substring $p..<$q), tags: $t}
    }
  } | compact
}

# Segments -> words with origin tags. A word straddling segments carries the
# union of its row tags; it is free only if every character is free.
def tokenize [segs: list] {
  mut words = []
  mut tags = []
  mut cur = ""
  mut ctags = []
  for s in $segs {
    let pieces = ($s.text | split row -r '\s+')
    let n = ($pieces | length)
    let st = ($s.tags | split row "," | where {|t| $t != "" })
    let head = ($pieces | first)
    if $head != "" {
      $cur = $cur + $head
      $ctags = ($ctags | append $st)
    }
    if $n > 1 {
      if $cur != "" {
        $words = ($words | append $cur)
        $tags = ($tags | append ($ctags | uniq | str join ","))
      }
      let mid = ($pieces | skip 1 | drop 1)
      let stj = ($st | str join ",")
      $words = ($words | append $mid)
      $tags = ($tags | append ($mid | each {|_| $stj }))
      $cur = ($pieces | last)
      $ctags = (if $cur != "" { $st } else { [] })
    }
  }
  if $cur != "" {
    $words = ($words | append $cur)
    $tags = ($tags | append ($ctags | uniq | str join ","))
  }
  {words: $words, tags: $tags}
}

def words-of [s: string] {
  $s | split row -r '\s+' | where {|w| $w != "" }
}

def ctx [ws: list, start: int, n: int] {
  let lo = ([0 ($start - $CTX)] | math max)
  let pre = ($ws | skip $lo | first ($start - $lo) | str join " ")
  let mid = ($ws | skip $start | first $n | str join " ")
  let post = ($ws | skip ($start + $n) | first $CTX | str join " ")
  ['…' $pre $"[($mid)]" $post '…'] | where {|s| $s != "" } | str join " "
}

def read-base [base: string, file: string] {
  let r = (do { ^git show $"($base):($file)" } | complete)
  if $r.exit_code == 0 { $r.stdout } else { null }
}

def read-post [root: string, tree: string, file: string] {
  let over = (if ($tree | is-empty) { "" } else { $tree | path join $file })
  if ($over != "") and ($over | path exists) { return (open --raw $over) }
  let p = ($root | path join $file)
  if ($p | path exists) { open --raw $p } else { null }
}

def main [
  --base: string      # pre-sweep commit (plugins/ install/ src/ equal to cdf79c1f8)
  --through: string   # cumulative unit scope: U2 | U3a | U3b | U3c | U4
  --applied: string   # explicit comma-separated applied ids (controls)
  --tree: string      # post-state overlay dir; <dir>/<path> wins over the working tree
] {
  if ($base | is-empty) { die "--base is required" }
  if ($through | is-empty) == ($applied | is-empty) { die "give exactly one of --through / --applied" }
  let root = (do { ^git -C $SLICE_DIR rev-parse --show-toplevel } | complete)
  if $root.exit_code != 0 { die "cannot resolve repo root" }
  let root = ($root.stdout | str trim)
  cd $root
  let bc = (do { ^git rev-parse --verify $"($base)^{commit}" } | complete)
  if $bc.exit_code != 0 { die $"--base ($base) is not a commit" }
  let tree = (if ($tree | is-empty) { "" } else { $tree | path expand })
  if ($tree != "") and not ($tree | path exists) { die $"--tree ($tree) does not exist" }

  let inv = ($SLICE_DIR | path join inventory.toml)
  let rows = (try { open $inv | get row } catch { die $"cannot read ($inv)" })
  if ($rows | is-empty) { die "inventory selects zero rows" }
  let ids = ($rows | get id)
  if ($ids | uniq | length) != ($ids | length) { die "duplicate row ids in inventory" }

  # ---- step 2: locate every row in base --------------------------------
  let rowfiles = ($rows | get file | uniq | sort)
  mut basetext = {}
  for f in $rowfiles {
    let t = (read-base $base $f)
    if $t == null { die $"cannot read ($base):($f), named by an inventory row" }
    $basetext = ($basetext | insert $f $t)
  }
  let bt = $basetext
  let located = ($rows | each {|r|
    let l = (locate ($bt | get $r.file) $r)
    $r | merge $l | insert changing (changes $r) | insert new (new-text $r)
  })

  # ---- scope: the applied set A ------------------------------------------
  let ch = ($located | where changing)
  for x in $ch {
    if not $x.ok { die $"changing row ($x.id) does not locate in base: ($x.why)" }
    if ($x.new | is-empty) { die $"changing row ($x.id) has empty new text" }
  }
  let chsub = ($ch | where {|x| $ch | any {|y| contains $y $x } } | get id)
  # design 6.3 unit ownership, by id block; a changing row nested in another
  # changing row is subsumed by it and owned by nobody.
  let owned = ($ch | where {|x| not ($x.id in $chsub) } | get id)
  let units = {
    U2: ($owned | where {|i| ($i | str starts-with "C-") and $i != "C-024" })
    U3a: ($owned | where {|i| $i | str starts-with "R-0" })
    U3b: ($owned | where {|i| $i | str starts-with "R-5" })
    U3c: ($owned | where {|i| $i | str starts-with "R-7" })
    U4: ($owned | where {|i| $i == "C-024" })
  }
  let partitioned = ($UNITS | each {|u| $units | get $u } | flatten)
  let expected_all = $owned
  if ($partitioned | uniq | length) != ($partitioned | length) { die "unit scopes overlap" }
  if ($partitioned | sort) != ($expected_all | sort) {
    die $"changing rows owned by no unit: ($expected_all | where {|i| not ($i in $partitioned) } | str join ',')"
  }
  let scope = (if ($through | is-not-empty) {
    let k = ($UNITS | enumerate | where item == $through)
    if ($k | is-empty) { die $"--through must be one of ($UNITS | str join '|')" }
    $UNITS | first (($k | first | get index) + 1) | each {|u| $units | get $u } | flatten
  } else {
    let want = ($applied | split row "," | each {|s| $s | str trim } | where {|s| $s != "" })
    for i in $want {
      if not ($i in $ids) { die $"--applied ($i): no such row" }
      if not ($i in ($ch | get id)) { die $"--applied ($i): not a changing row under the Outcome rule" }
    }
    $want | uniq
  })
  if ($scope | is-empty) { die "scope selects zero rows" }
  let a0 = ($located | where {|x| $x.id in $scope })
  # an applied row nested in another applied row is subsumed by it
  let nested = ($a0 | where {|x| $a0 | any {|y| contains $y $x } })
  let arows = ($a0 | where {|x| not ($x.id in ($nested | get id)) })
  for x in $arows {
    for y in $arows {
      if $x.id < $y.id and $x.file == $y.file and $x.a < $y.b and $y.a < $x.b {
        die $"applied spans overlap without nesting: ($x.id) and ($y.id)"
      }
    }
  }
  let aids = ($arows | get id)
  let subsumed = ($located | where {|x| not ($x.id in $aids) and ($arows | any {|y| contains $y $x }) })
  let subids = ($subsumed | get id)

  mut info = []
  mut viol = []
  for x in $nested { $info = ($info | append $"INFO nested: applied ($x.id) lies inside another applied span; subsumed") }

  # ---- inventory drift vs base (I5): inventory.toml is the run-time truth, so
  # every amend, added row or field change since --base is disclosed; a row
  # removed since base is a violation (ids are append-only).
  let invrel = ($inv | path relative-to $root)
  let binv = (read-base $base $invrel)
  if $binv == null { die $"cannot read ($base):($invrel)" }
  let brows = (try { $binv | from toml | get row } catch { die $"cannot parse ($base):($invrel)" })
  mut drift = 0
  for r in $rows {
    let b = ($brows | where id == $r.id)
    if ($b | is-empty) {
      $info = ($info | append $"INVENTORY ($r.id): row added since base \(verdict ($r.verdict), recommend ($r.recommend)\)")
      $drift = $drift + 1
    } else {
      let b = ($b | first)
      let cols = ($r | columns | append ($b | columns) | uniq | where {|c| ($r | get -o $c) != ($b | get -o $c) })
      if ($cols | is-not-empty) {
        let vd = (if "verdict" in $cols { $" [verdict '($b | get -o verdict)' -> '($r.verdict)']" } else { "" })
        $info = ($info | append $"INVENTORY ($r.id): changed since base: ($cols | str join ',')($vd)")
        $drift = $drift + 1
      }
    }
  }
  for b in ($brows | where {|b| not ($b.id in $ids) }) {
    $viol = ($viol | append {kind: INVENTORY-REMOVED, file: $invrel, rows: [$b.id], detail: "row present at base is gone"})
  }

  # ---- step 3 INFO: containment vs the advisory `within` field ------------
  let rrows = ($located | where {|x| $x.id | str starts-with "R-" })
  for x in ($located | where {|x| ($x.id | str starts-with "C-") and $x.ok }) {
    let inr = ($rrows | where {|y| contains $y $x } | get id)
    if ($x.within != "") and not ($x.within in $inr) {
      $info = ($info | append $"INFO within: ($x.id) has within=($x.within) but its span lies outside ($x.within)'s span")
    } else if ($inr | is-not-empty) and not ($x.within in $inr) {
      $info = ($info | append $"INFO within: ($x.id) lies inside ($inr | str join ',') but within='($x.within)'")
    }
  }

  for x in ($located | where {|x| not $x.ok }) {
    $viol = ($viol | append {kind: LOCATE, file: $x.file, rows: [$x.id], detail: $x.why})
  }

  # ---- step 1: the file set ----------------------------------------------
  let changed = (do { ^git diff --name-only $base -- ...$ROOTS } | complete)
  if $changed.exit_code != 0 { die "git diff failed" }
  let untracked = (do { ^git ls-files --others --exclude-standard -- ...$ROOTS } | complete)
  if $untracked.exit_code != 0 { die "git ls-files failed" }
  let overlay = (if $tree == "" { [] } else {
    glob $"($tree)/**/*" | where {|p| ($p | path type) == file } | each {|p| $p | path relative-to $tree }
  })
  for f in $overlay {
    if not ($ROOTS | any {|r| $f | str starts-with $"($r)/" }) { die $"overlay file outside the shipped roots: ($f)" }
  }
  let files = ($rowfiles | append ($changed.stdout | lines) | append ($untracked.stdout | lines) | append $overlay | where {|f| $f != "" } | uniq | sort)

  let tmp = (^mktemp -d | str trim)
  mut rowstat = {}
  mut width = []
  mut quick = 0
  for f in $files {
    let base_t = (if $f in $rowfiles { $bt | get $f } else { read-base $base $f })
    let post_t = (read-post $root $tree $f)
    if $base_t == null {
      $viol = ($viol | append {kind: NEW-FILE, file: $f, rows: [], detail: "file absent at base; no row can authorise it"})
      continue
    }
    let frows = ($located | where file == $f)
    if $post_t == null {
      $viol = ($viol | append {kind: DELETED-FILE, file: $f, rows: ($frows | get id), detail: "file unreadable in post-state"})
      continue
    }
    let fa = ($arows | where file == $f)
    if ($fa | is-empty) and ($post_t == $base_t) {
      # expected == base == post: every located row in the file is unchanged
      for x in ($frows | where ok) {
        if not ($x.id in $subids) { $rowstat = ($rowstat | insert $x.id unchanged-ok) }
      }
      $quick = $quick + 1
      continue
    }
    let tsp = ($frows | where {|x| $x.ok and not ($x.id in $aids) and not ($x.id in $subids) })
    for x in $tsp {
      for y in $fa {
        if $x.a < $y.b and $y.a < $x.b {
          $viol = ($viol | append {kind: OVERLAP, file: $f, rows: [$x.id $y.id], detail: $"($x.id)'s span straddles applied ($y.id)'s span"})
        }
      }
    }
    let segs = (segments $base_t 0 ($base_t | str length) $fa $tsp)
    let exp = (tokenize $segs)
    let post_w = (words-of $post_t)
    let ef = ($tmp | path join "exp")
    let pf = ($tmp | path join "post")
    ($exp.words | str join "\n") + "\n" | save -f $ef
    ($post_w | str join "\n") + "\n" | save -f $pf
    let d = (do { ^diff -U0 $ef $pf } | complete)
    if $d.exit_code > 1 { die $"diff failed on ($f): ($d.stderr)" }
    let hunks = ($d.stdout | lines | where {|l| $l | str starts-with "@@" }
      | parse -r '^@@ -(?<es>\d+)(?:,(?<en>\d+))? \+(?<ps>\d+)(?:,(?<pn>\d+))? @@')
    let nw = ($exp.words | length)
    mut fh = 0
    for h in $hunks {
      let es = ($h.es | into int)
      let en = (if $h.en == null { 1 } else { $h.en | into int })
      let ps = ($h.ps | into int)
      let pn = (if $h.pn == null { 1 } else { $h.pn | into int })
      let idx = (if $en > 0 { ($es - 1)..<($es - 1 + $en) | each {|i| $i } } else {
        [($es - 1) $es] | where {|i| $i >= 0 and $i < $nw }
      })
      let tg = ($idx | each {|i| $exp.tags | get $i })
      let hr = ($tg | each {|t| $t | split row "," } | flatten | where {|t| $t != "" } | uniq)
      let has_free = ($en > 0) and ($tg | any {|t| $t == "" })
      let foreign = ($hr | where {|i| not ($i in $aids) })
      let kind = (if ($hr | is-empty) or ($foreign | is-not-empty) or $has_free { "UNAUTHORISED" } else { "MISAPPLIED" })
      let desc = ($hr | each {|i|
        let r = ($located | where id == $i | first)
        if $i in $aids { $"($i) \(applied\)" } else { $"($i) \(verdict ($r.verdict), recommend ($r.recommend)\)" }
      } | append (if $has_free or ($hr | is-empty) { ["free text"] } else { [] }) | str join "; ")
      let e0 = (if $en > 0 { $es - 1 } else { $es })
      let p0 = (if $pn > 0 { $ps - 1 } else { $ps })
      $viol = ($viol | append {
        kind: $kind, file: $f, rows: $hr,
        detail: $"($desc)\n    expected: (ctx $exp.words $e0 $en)\n    post:     (ctx $post_w $p0 $pn)"
      })
      $fh = $fh + 1
    }
    # ---- step 6: per-row status, cross-checked on normalised excerpts ------
    let npost = (norm $post_t)
    let named = ($viol | where file == $f | get rows | flatten | uniq)
    for x in ($frows | where {|x| $x.ok and not ($x.id in $subids) }) {
      let s = (segments $base_t $x.ea $x.eb $fa [])
      let st = (if $x.id in $named {
        if $x.id in $aids { "MISAPPLIED" } else { "CHANGED" }
      } else if $s == null {
        "ok-diff-only"
      } else if ($npost | str contains (norm ($s | get text | str join ""))) {
        if $x.id in $aids { "OK" } else { "unchanged-ok" }
      } else if $fh == 0 {
        $viol = ($viol | append {kind: CROSSCHECK, file: $f, rows: [$x.id], detail: "word diff is clean but the row's expected excerpt is absent from the post text"})
        "CROSSCHECK"
      } else {
        "EXCERPT-TOUCHED"
      })
      $rowstat = ($rowstat | insert $x.id $st)
    }
    if ($post_t != $base_t) and ($fa | is-empty) and ($fh == 0) {
      $viol = ($viol | append {kind: UNAUTHORISED-FILE, file: $f, rows: [], detail: "file changed (whitespace only) but holds no applied row"})
    }
    # ---- step 8 INFO: over-width lines not identical to a base line --------
    if $post_t != $base_t {
      let bl = ($base_t | lines)
      for l in ($post_t | lines | enumerate) {
        let w = ($l.item | str length -g)
        if $w > $WIDTH and not ($l.item in $bl) {
          $width = ($width | append $"WIDTH ($f):($l.index + 1) \(($w) cols\)")
        }
      }
    }
  }
  rm -rf $tmp

  # ---- step 7: IMP-500 carries every non-rejected log row ------------------
  let logrows = ($located | where {|x| $x.recommend == "log" and $x.verdict != "reject" })
  let bin = ($root | path join target/debug/doctrine)
  if not ($bin | path exists) { die $"($bin) missing; run cargo build" }
  let shown = (do { ^$bin show $LOG_ENTITY } | complete)
  if $shown.exit_code != 0 { die $"doctrine show ($LOG_ENTITY) failed" }
  for x in $logrows {
    if not ($shown.stdout | str contains $"($x.file):($x.line)") {
      $viol = ($viol | append {kind: LOG-MISSING, file: $x.file, rows: [$x.id], detail: $"($LOG_ENTITY) lacks ($x.file):($x.line)"})
    }
  }

  # ---- report ----------------------------------------------------------------
  let rs = $rowstat
  let scope_name = (if ($through | is-not-empty) { $"--through ($through)" } else { $"--applied ($applied)" })
  print $"base ($base) · ($scope_name) · tree (if $tree == '' { 'worktree' } else { $tree })"
  print $"partition: changing ($ch | length) = unit-applied ($partitioned | length) + subsumed-by-changing ($chsub | length) · (($UNITS | each {|u| $'($u) ($units | get $u | length)' }) | str join ' · ')"
  print $"files: ($files | length) checked, ($quick) byte-identical to base with no applied row"
  for i in $info { print $i }
  for v in $viol { print $"($v.kind) ($v.file) [($v.rows | str join ',')]: ($v.detail)" }
  let bad = ($located | where {|x| $x.ok and not ($x.id in $subids) } | where {|x|
    let s = ($rs | get -o $x.id)
    $s == null or not ($s in [OK unchanged-ok ok-diff-only])
  })
  for x in $bad { print $"ROW ($x.id) ($x.file): (($rs | get -o $x.id) | default 'UNCHECKED')" }
  for w in $width { print $w }
  let st = ($located | each {|x| $rs | get -o $x.id })
  let unch = ($st | where {|s| $s == "unchanged-ok" } | length)
  let donly = ($st | where {|s| $s == "ok-diff-only" } | length)
  let okn = ($st | where {|s| $s == "OK" } | length)
  let kinds = ($viol | group-by kind | transpose k v | each {|g| $"($g.k) ($g.v | length)" } | str join ", ")
  let nv = ($viol | length)
  print $"rows ($rows | length) · applied ($aids | length) \(OK ($okn)\) · subsumed ($subids | length) · unchanged-ok ($unch) · diff-only ($donly) · row-failures ($bad | length) · violations ($nv)(if $nv > 0 { " (" + $kinds + ")" } else { "" }) · inventory-drift ($drift) · width ($width | length)"
  if $nv > 0 or ($bad | is-not-empty) { exit 1 }
}
