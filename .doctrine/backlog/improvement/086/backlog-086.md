# IMP-086: Pin web/map vendor dependencies with version metadata and SRI integrity hashes

**Origin**: RV-049 F-10 (code-review of IMP-085)

`web/map/vendor/` contains minified libraries (markdown-it, DOMPurify,
github-markdown.css) with no version numbers in filenames, no SRI hashes, no
lockfile. There is no automated path to determine whether a published CVE affects
this installation.

## Options

- **SRI + version comment**: Add `integrity` hashes to `<script>` tags, rename
  files to include version (e.g. `markdown-it-14.1.0.min.js`), add a
  `vendor/README.md` with provenance URLs.
- **Download script**: A `vendor/update.sh` that fetches from CDN with pinned
  versions and verifies hashes.
- **npm + copy**: A `package.json` with pinned versions, postinstall copier, and
  `npm audit` in CI. Overkill for 3 files; only if the vendor set grows.
