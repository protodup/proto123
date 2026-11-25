# Proto123 Trademark-Safe Feature Names & Terminology

> **Policy Reference:** `AGENTS.md` Rules 17, 23 & 24 (Clean-Room & Trademark-Safe Naming)
> **Scope:** All user-visible UI labels, Slash Command Bar (`/`) menus, dialog text, CLI `--help` output, error messages, and documentation headings.

**Proto123** is inspired by the speed and clarity of classic terminal spreadsheets and adheres to the open **OASIS ODF 1.3 / OpenFormula** standard. To respect third-party trademarks and keep our clean-room identity unambiguous, user-visible feature names MUST use our own clear terminology rather than trademarked product or feature names owned by other companies.

---

## 1. Core Naming Rules

1. **Do NOT brand Proto123 features with third-party product trademarks** (such as "Lotus 1-2-3", "Microsoft Excel", "Google Sheets", or "LibreOffice Calc") in menus, dialogs, status bars, CLI flags, or feature headings.
2. **External format and interoperability mentions are allowed factually:** Mentioning `.ods` (OpenDocument Spreadsheet), `OASIS ODF 1.3`, `OpenFormula`, or external suites in factual compatibility notes (e.g., *"Exports standard ODF 1.3 `.ods` files compatible with LibreOffice Calc"*) is acceptable in documentation—never as the name of a Proto123 feature.
3. **Normative OpenFormula function names stay standard:** Function identifiers defined by OASIS ODF 1.3 Part 4 (OpenFormula)—such as `SUM`, `AVERAGE`, `VLOOKUP`, `INDEX`, `MATCH`, `IFERROR`—are part of an international open standard (ISO/IEC 26300) and MUST be preserved verbatim inside formula syntax. Use generic descriptive names only for UI labels or docs titles describing them.

---

## 2. Approved Terminology Mapping

| Avoid Using as a Feature Name | Approved Proto123 Term | Notes |
|---|---|---|
| Lotus 1-2-3 / 1-2-3 Clone | **Proto123** / **Classic TUI Spreadsheet** | Product and category name |
| Lotus `/` Menu / 1-2-3 Menu Bar | **Slash Command Bar** (or **Command Menu**) | Hierarchical two-line top command bar triggered by `/` or mouse click |
| Lotus `@`-Functions | **At-Prefix Shorthand** / **OpenFormula Functions** | Standard `=` syntax is canonical; optional `@` input shorthand maps to OpenFormula AST |
| Excel Formulas | **OpenFormula Expressions** | Refers to `of:=...` expressions per OASIS ODF 1.3 Part 4 |
| Excel Workbook | **Workbook** / **ODS Workbook** | Native in-memory and `.ods` document container |
| PivotTable | **Cross-Tab Summary** (or **Summary Table**) | Avoid Microsoft's trademarked term |
| AutoSum | **Quick Sum** | One-action sum over adjacent numeric cells |
| AutoFill | **Series Fill** | Extending patterns/sequences across a range |
| Format Painter | **Style Copy** | Copying cell format attributes to another range |
| Freeze Panes | **Locked Headers** (or **Titles**) | Locking top rows / left columns during scroll (`/Worksheet/Titles`) |
| 1-2-3 Macros / VBA Macros | **Command Script** / **Headless Command Batch** | Scripted `CommandId` JSON sequences run via `proto123-cli exec` |
| Conditional Formatting *(generic, OK)* | **Conditional Styling** / **Conditional Format** | Generic descriptive term |

---

## 3. Canonical TUI Mode Indicator Strings

The top-right mode badge MUST use these exact uppercase tokens (and mirror them to the web host's ARIA live region):

| Token | Meaning |
|---|---|
| `READY` | Idle grid navigation (keyboard or mouse) |
| `VALUE` | Entering numeric literal or formula starting with `=`, `+`, `-`, `.`, or digit |
| `LABEL` | Entering text string (with optional alignment prefix `'`, `"`, `^`) |
| `EDIT` | Editing existing cell content in the formula bar (`F2` or double-click) |
| `POINT` | Selecting live cell/range references by keyboard or mouse during formula entry |
| `MENU` | Navigating the Slash Command Bar (`/`) |
| `DIALOG` | Interacting with a modal terminal prompt (e.g., Fetch URL, Save filename) |
| `CALC` | Active recalculation indicator (for very large DAG updates) |
| `ERROR` | Recoverable command or I/O error awaiting acknowledgement (`Esc`/`Enter`) |

---

## 4. Canonical Menu Top-Level Labels

Top-level entries in `menu_catalog.rs` SHOULD stay concise (one word, unique first letter for single-key access, and full mouse hover/click targets):

- `File` — New, Open, Fetch URL, Save ODS, Push Server, Validate ODF
- `Edit` — Undo, Redo, Copy, Cut, Paste
- `Range` — Erase, Format, Label, Sort, Goto
- `Worksheet` — Insert, Delete, Column, Sheet, Titles, Recalc
- `View` — Theme, Show Formulas, Grid Borders
- `Help` — Shortcuts, Mouse Controls, About Proto123

---

## 5. Review Checklist for PRs

Before merging any PR that introduces UI copy, menu entries, CLI flags, or documentation:

1. Verify all new UI labels live in the centralized English string catalog (`AGENTS.md` Rule 17).
2. Confirm no third-party trademarks appear as Proto123 feature names, CLI subcommands, or error messages.
3. Confirm every top-level and submenu label in `menu_catalog.rs` has an unambiguous single-letter keyboard trigger and fits cleanly within an 80-column TUI viewport without truncation.