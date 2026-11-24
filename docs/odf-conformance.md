# Classic123 ODF 1.3 Conformance Specification

> **Standard Target:** OASIS OpenDocument Format (ODF) 1.3 / ISO/IEC 26300 (Parts 2, 3, and 4 — Packages, OpenDocument Schema, and Recalculated Formula / OpenFormula)
> **Owning Crates:** `classic123-ods` (L3), `classic123-xlsx` and `classic123-csv` (L3, import only), `classic123-formula` (L1), `classic123-value` (L0)
> **Automated Gate:** `cargo xtask ods-conformance`

This document defines the exact structural, XML, OpenFormula, and security invariants that **Classic123** MUST uphold so every generated `.ods` file opens cleanly without repair prompts, formula loss, or semantic drift in LibreOffice Calc, Collabora, and other ODF-compliant suites.

**Output is always `.ods`.** `.xlsx` and `.csv` are accepted as input only (see Section 9); Classic123 never writes them.

---

## 1. ZIP Package Container Rules (ODF 1.3 Part 2)

Every `.ods` archive written by `classic123-ods` MUST conform strictly to the ODF ZIP container layout:

1. **Entry #0 (`mimetype`):**
   - MUST be the **first** entry in the ZIP archive.
   - MUST be stored uncompressed (`CompressionMethod::Stored`).
   - MUST NOT contain ZIP extra fields, encryption, or a trailing newline.
   - MUST contain the exact 46 ASCII bytes:
     ```text
     application/vnd.oasis.opendocument.spreadsheet
     ```
2. **Required Package Files:**
   - `content.xml` — spreadsheet tables, cell values, cached display text, and OpenFormula expressions.
   - `styles.xml` — named styles, default number/date/currency formats, and page layout definitions.
   - `meta.xml` — document metadata (generator `Classic123/<version>`, creation/modification timestamps in ISO 8601 UTC via `chrono`).
   - `META-INF/manifest.xml` — package manifest declaring `manifest:version="1.3"`, the root entry (`manifest:full-path="/"` with media type `application/vnd.oasis.opendocument.spreadsheet`), and every XML file present in the archive.
3. **Deterministic Serialization:**
   - Entry order, XML attribute order, and style IDs MUST be deterministic across native and `wasm32-unknown-unknown` builds so identical workbooks produce reproducible `.ods` output.

---

## 2. Normative XML Namespaces

`content.xml`, `styles.xml`, `meta.xml`, and `META-INF/manifest.xml` MUST declare and use standard OASIS ODF 1.3 namespace URIs:

| Prefix | Namespace URI |
|---|---|
| `office` | `urn:oasis:names:tc:opendocument:xmlns:office:1.0` |
| `table` | `urn:oasis:names:tc:opendocument:xmlns:table:1.0` |
| `text` | `urn:oasis:names:tc:opendocument:xmlns:text:1.0` |
| `style` | `urn:oasis:names:tc:opendocument:xmlns:style:1.0` |
| `fo` | `urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0` |
| `number` | `urn:oasis:names:tc:opendocument:xmlns:datastyle:1.0` |
| `meta` | `urn:oasis:names:tc:opendocument:xmlns:meta:1.0` |
| `dc` | `http://purl.org/dc/elements/1.1/` |
| `of` | `urn:oasis:names:tc:opendocument:xmlns:of:1.2` |
| `manifest` | `urn:oasis:names:tc:opendocument:xmlns:manifest:1.0` |

Root elements MUST carry `office:version="1.3"`.

---

## 3. Cell Value Types & Mapping (`classic123-value` <-> ODF XML)

Every populated `<table:table-cell>` MUST carry both the machine-readable ODF value attributes and the cached display representation inside `<text:p>` so viewers that do not recalculate on open render the exact evaluated result:

| Internal `CellValue` | `office:value-type` | Machine Value Attribute(s) | Cached Display (`<text:p>`) |
|---|---|---|---|
| `Number(f64)` | `float` | `office:value="<ieee-decimal>"` | Formatted numeric string |
| `Currency(f64, code)` | `currency` | `office:value="<decimal>"`, `office:currency="USD"` | Formatted currency string |
| `Percentage(f64)` | `percentage` | `office:value="<ratio>"` (e.g., `0.25`) | Formatted percent (e.g., `25%`) |
| `Date(NaiveDate)` | `date` | `office:date-value="YYYY-MM-DD"` | Formatted date string |
| `DateTime(NaiveDateTime)` | `date` | `office:date-value="YYYY-MM-DDTHH:MM:SS"` | Formatted date-time string |
| `Duration` / `Time` | `time` | `office:time-value="PT...S"` (ISO 8601 duration) | Formatted duration string |
| `Bool(bool)` | `boolean` | `office:boolean-value="true"` or `"false"` | `TRUE` / `FALSE` |
| `Text(String)` | `string` | *(optional `office:string-value` on formulas)* | Escaped `<text:p>` lines |
| `Error(CalcError)` | `string` | `office:string-value="#DIV/0!"` (etc.) | Error literal (`#DIV/0!`, `#REF!`, etc.) |
| `Empty` | *(omitted)* | *(omitted)* | *(omitted)* |

- Non-finite IEEE-754 numbers (`NaN`, `±Inf`) MUST NEVER be serialized as raw `office:value="NaN"` or `"inf"`; they MUST evaluate to `#NUM!` or `#DIV/0!` before serialization.
- Date/time conversions MUST use `chrono`—never hand-rolled calendar arithmetic.

---

## 4. Sparse Sheet Compression

Because `classic123-model` uses sparse storage:

- Consecutive identical empty cells MUST be collapsed using `table:number-columns-repeated`.
- Consecutive empty rows MUST be collapsed using `table:number-rows-repeated`.
- Trailing empty cells in a row and trailing empty rows in a sheet MUST NOT be emitted unnecessarily.
- On **read**, `classic123-ods` MUST ignore excessive trailing empty repetition counts (e.g., `table:number-columns-repeated="16384"` or `table:number-rows-repeated="1048576"` emitted by external suites for default formatting) without allocating memory for empty cells.

---

## 5. OpenFormula (`of:=...`) Conformance (ODF 1.3 Part 4)

### 5.1 Reference & Expression Syntax
- All formula cells MUST store formulas in `table:formula` prefixed with `of:=` (e.g., `table:formula="of:=SUM([.A1:.A10])"`).
- **Same-sheet cell reference:** `[.A1]`, absolute `[$A$1]` or `[.$A$1]`.
- **Same-sheet range reference:** `[.A1:.C10]`.
- **Cross-sheet reference:** `[Sheet2.B4]` or `['Budget 2026'.B4]` (single-quoted when the sheet name contains spaces or special characters, with internal `'` doubled as `''`).
- **String literals** inside OpenFormula use double quotes (`"text"`), with embedded `"` escaped as `""`.

### 5.2 Error Literals
Error values follow OASIS OpenFormula conventions and MUST propagate cleanly without panics or silent fallback to `0`:

| Literal | Meaning |
|---|---|
| `#DIV/0!` | Division by zero |
| `#VALUE!` | Wrong operand or argument type |
| `#REF!` | Invalid, deleted, or out-of-bounds cell/sheet reference |
| `#NAME?` | Unrecognized function name or identifier |
| `#NUM!` | Invalid numeric domain (e.g., `SQRT(-1)`, overflow) |
| `#N/A` | Value not available (lookup miss or explicit `NA()`) |
| `#CIRC!` | Circular dependency detected by `classic123-calc` DAG |
| `#ERR` | General parser or syntax error |

### 5.3 Function Coverage Tiers
- **Tier 1 (Phase 1 MVP):** Arithmetic (`+`, `-`, `*`, `/`, `^`, unary `-`, `%`), comparisons (`=`, `<>`, `<`, `<=`, `>`, `>=`), concatenation (`&`), `SUM`, `AVERAGE`, `MIN`, `MAX`, `COUNT`, `COUNTA`, `IF`, `AND`, `OR`, `NOT`, `ABS`, `ROUND`, `INT`, `SQRT`, `MOD`.
- **Tier 2 (Phase 3–4 Core Office Parity):** `SUMIF`, `COUNTIF`, `IFERROR`, `ISBLANK`, `ISNUMBER`, `ISTEXT`, `VLOOKUP`, `HLOOKUP`, `INDEX`, `MATCH`, `LEFT`, `RIGHT`, `MID`, `LEN`, `TRIM`, `UPPER`, `LOWER`, `CONCATENATE`, `DATE`, `YEAR`, `MONTH`, `DAY`, `TODAY`, `NOW`, `NPV`, `PMT`, `FV`, `PV`.
- **Unsupported Functions:** If an imported `.ods` contains a valid OpenFormula function not yet implemented in `classic123-formula`, the original formula AST/text MUST be preserved for round-trip while evaluating to an explicit `#NAME?` diagnostic (or preserving the cached value only in read-only inspect mode with an explicit warning—never silently claiming evaluation succeeded).

---

## 6. Round-Trip Guarantees & Unknown Metadata

When loading an existing `.ods` and saving it back:

- **Guaranteed preserved:** sheet names & order, cell coordinates, values, types, OpenFormula expressions, column widths, basic alignment & number styles, and document metadata.
- **Pass-through preservation:** unrecognized XML styles, custom namespaces, or non-tabular metadata SHOULD be retained in an opaque per-workbook/per-sheet extension bag where safe, or explicitly flagged in `classic123-cli inspect` diagnostics when dropped.

---

## 7. Hostile-Input & Safety Guards (Zero-Panic Policy)

Because `classic123-ods` runs directly on untrusted remote URLs and local user files, inside WebAssembly and in the native terminal app:

- **Decompression Bomb Limit:** Enforce a strict maximum uncompressed byte budget per XML entry and per archive (configurable, default 64 MiB for browser WASM).
- **XML Depth & Entity Guards:** Reject deeply nested XML (default max depth 128) and forbid external DTD/XXE expansion (`quick-xml` does not expand external entities).
- **Coordinate Bounds:** Clamp/reject references exceeding ODF/modern sheet bounds (`1,048,576` rows × `16,384` columns) with a typed `OdsError::CoordinateOutOfBounds`, never integer overflow or panic.
- **Invalid UTF-8 / Truncated ZIPs:** Always return structured `Result<Workbook, OdsError>`.

---

## 8. Conformance Verification (`cargo xtask ods-conformance`)

Every PR touching `classic123-ods`, `classic123-formula`, or `classic123-value` MUST pass `cargo xtask ods-conformance`, which verifies:

1. **ZIP Structure Validator (`classic123-testkit`):** Asserts `mimetype` is Entry #0, `STORED`, no extra fields, exact bytes, and `META-INF/manifest.xml` lists every archive part.
2. **XML & Namespace Validator:** Confirms well-formed XML, `office:version="1.3"`, and valid `of:=` formula prefixes.
3. **Round-Trip Invariant Suite (`corpus/valid/`):** `Workbook -> write_ods -> read_ods -> recalc` matches all cell values, types, and canonical formulas.
4. **Hostile Corpus Sweep (`corpus/hostile/`):** Truncated ZIPs, compressed `mimetype`, wrong entry order, cyclic formulas, and extreme repeated-row counts fail cleanly with zero panics.
5. **Import Round-Trip:** `.xlsx`/`.csv` fixtures in `corpus/valid/` import, save as `.ods`, re-read, and recalculate with matching values, types, and sheets.

---

## 9. Import-Only Formats (`.xlsx`, `.csv`)

- **`.xlsx` (`classic123-xlsx`):** Reads sheets, typed values (number, string, boolean, date), and formulas, translating formulas to the canonical AST so they save as OpenFormula. A formula that cannot be translated keeps its cached value and raises an explicit import diagnostic—never a silent substitution.
- **`.csv` (`classic123-csv`):** Reads RFC 4180 CSV/TSV into the active or a new sheet, with delimiter and encoding guards.
- **Same safety guards:** zip-bomb limits, XML depth limits, coordinate bounds, and typed errors apply to `.xlsx` exactly as to `.ods`.
- **Saving:** an imported workbook is always saved as ODF 1.3 `.ods`.