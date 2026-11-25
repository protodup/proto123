# Contract: `cargo xtask` Subcommands (Spec 001 subset)

Alias in `.cargo/config.toml`: `xtask = "run -q -p xtask --"`. Every subcommand prints one line per finding and exits `0` on success, `1` on any violation, `2` on usage/tooling error (e.g. `trunk` or `docker` missing — reported, never treated as a pass).

| Command | Does | Fails when |
|---|---|---|
| `layers` | `cargo metadata` → `layers::check` (research R12) | unregistered member; dependency on a higher layer (prints `crate (Lx) -> dep (Ly)`); `proto123-testkit` outside dev-deps; UI/browser/terminal crate below its allowed layer |
| `menu-commands` | `proto123_engine::menu::verify(CATALOG, REGISTRY)` | leaf id not registered (prints menu path + id); duplicate/mismatched trigger; missing string key |
| `wasm` | `cargo build --target wasm32-unknown-unknown` for each L0–L5 crate; `trunk build --release`; report `dist/*.wasm` raw bytes and gzip-9 bytes | any build fails; no `.wasm` produced |
| `docker-check` | `docker build -t proto123:check .`; run on a free port; GET `/` and the `.wasm` path; stop container | build/run fails; non-200; `.wasm` `Content-Type` ≠ `application/wasm` |
| `ci` | runs `fmt --check`, `clippy --workspace --all-targets -D warnings`, `layers`, `menu-commands`, `test --workspace`, `wasm` in order | first failing step (named) |

Deferred to later specs (listed in docs, not built here): `ods-conformance`, `parity` generation, `stats`, `test` (nextest wrapper). `docs/parity-checklist.md` is updated by hand in this slice for the commands above.

Size report format (stdout, also used as the WASM size baseline):

```text
wasm: dist/proto123-wasm-<hash>_bg.wasm raw=<bytes> gzip=<bytes>
```
