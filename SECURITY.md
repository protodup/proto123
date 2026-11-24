# Security Policy

## Supported Versions

**Classic123** (`rusted-classic-123`) is currently in active pre-1.0 development. Security fixes are applied directly to the `main` branch. Once tagged releases begin, the latest minor release line will receive security patches.

---

## Reporting a Vulnerability

**Please DO NOT report security vulnerabilities in public GitHub issues, pull requests, or discussions.**

Instead, report vulnerabilities privately through **GitHub Security Advisories** ("Report a vulnerability" under the repository's **Security** tab). Please include:

- A clear description of the vulnerability and affected crate/component (e.g., `classic123-ods`, `classic123-formula`, `classic123-cli`, or `web/`).
- Minimal reproduction steps or a proof-of-concept input (e.g., a malformed `.ods` archive, formula string, or CLI command).
- Expected vs. actual behavior and potential impact (panic/DoS in WASM, unbounded memory allocation, path traversal, XSS/injection in web host).

We aim to acknowledge private reports promptly and coordinate a fix together with a regression entry in `corpus/hostile/` before public disclosure.

---

## Threat Model & Architectural Defenses

Although Classic123 runs 100% client-side in the browser (with no backend calculation server), it parses untrusted binary archives (`.ods`), text files (`.csv`), remote URLs, and formulas. The project enforces the following defense-in-depth boundaries:

### 1. Memory Safety & Zero-Panic Policy
- Workspace root enforces `unsafe_code = "forbid"`.
- Non-test code MUST NOT panic on untrusted inputs (`unwrap`, `expect`, unchecked slice indexing). Every malformed archive, formula, or coordinate returns a typed `Result` or spreadsheet error (`#ERR`, `#REF!`, `#CIRC!`).

### 2. Untrusted `.ods` Archives & XML (`classic123-ods`)
- **ZIP Decompression Bombs:** Strict per-entry and total uncompressed byte budgets (default 64 MiB in WASM) abort expansion before exhausting browser memory.
- **Sparse Sheet Exhaustion:** Excessive `table:number-rows-repeated` or `table:number-columns-repeated` counts on empty cells are skipped without allocating memory; populated coordinates are strictly bounded (`1,048,576` rows × `16,384` columns).
- **XML Safety:** Uses `quick-xml` without external DTD/entity expansion (immune to XXE) and enforces a maximum element nesting depth (default 128).
- **Formula DoS & Cycles:** Parser enforces maximum AST recursion depth; `classic123-calc` detects cyclic references deterministically via topological sort (`#CIRC!`) without stack overflow or infinite loops.

### 3. Browser Host & Zero-DOM Rendering (`web/`, `apps/classic123-wasm`)
- **XSS Mitigation:** Spreadsheet cell values and formula strings are rendered exclusively as monospace glyphs on a `<canvas>` surface and mirrored to the ARIA live region via `textContent` (never `innerHTML`).
- **Remote File Loading (`?file=...`):** The thin JS host fetches bytes only from URLs allowed by the hosting page's origin/CORS policy and Content-Security-Policy (CSP). Site operators deploying Classic123 SHOULD restrict allowed remote origins if loading untrusted query parameters (see `docs/web-integration.md`).
- **No Secrets in Client Bundles:** Never embed API keys, bearer tokens, or private server credentials in `web/` static files or test fixtures.

### 4. Headless CLI Path Sandboxing (`apps/classic123-cli`)
- Scripted/automated execution modes (`classic123-cli exec`) enforce explicit `--read-root` and `--write-root` directory boundaries, canonicalizing paths and rejecting `..` traversal or symlink escapes outside permitted roots.

---

## Continuous Security Testing

- Hostile-input sweeps (`cargo test -p classic123-engine --test panic_hunt -- --ignored` and `cargo xtask ods-conformance`) continuously test corrupt ZIPs, malformed XML, cyclic formulas, and extreme coordinates.
- Every crash, panic, infinite loop, or memory-exhaustion bug fix MUST add its triggering input to `corpus/hostile/`.