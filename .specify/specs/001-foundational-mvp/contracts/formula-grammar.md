# Contract: Entry Classification, Formula Grammar, and Number Display

## 1. Entry classification (FR-004, FR-005, FR-015) — `proto123-engine`

Applied to the committed buffer, in order:

1. Trimmed text matches **plain number** `^[+-]?(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?$` → `CellContent::Number` (must be finite, else diagnostic).
2. First char is `'`, `"`, or `^` → `Label { prefix, align: Left/Right/Center, text: rest }`.
3. First char ∈ `= + - . ( @` or an ASCII digit → **formula**: strip a leading `=`, `+`, or `@`-function shorthand, parse the rest. Parse error → not committed; status `diag.syntax` with char position; if the entry did not start with `=`, `+`, or `@` also `diag.useQuoteForText` (e.g. `123 Main St`, `1,000`).
4. Anything else → `Label { prefix: None, align: Left }`.
5. Empty buffer → erase the cell (counts as an edit; one undo step).

## 2. Grammar (EBNF; whitespace allowed between tokens)

```
formula   = expr ;
expr      = term   { ("+" | "-") term } ;            (* left-assoc, bp 10 *)
term      = power  { ("*" | "/") power } ;            (* left-assoc, bp 20 *)
power     = unary  { "^" unary } ;                    (* LEFT-assoc, bp 30: 2^3^2 = 64 *)
unary     = ("-" | "+") unary | primary ;             (* binds tighter than ^: -2^2 = 4 *)
primary   = number | "TRUE" | "FALSE" | errorlit | range | ref | call | "(" expr ")" ;
call      = ["@"] name "(" [ arg { ("," | ";") arg } ] ")" ;
arg       = expr ;                                    (* expr includes ref and range *)
range     = ref ":" ref ;
ref       = ["$"] letters ["$"] digits ;              (* 1–3 letters, case-insensitive *)
name      = letter { letter | digit | "." | "_" } ;   (* case-insensitive; stored upper-case *)
number    = digits ["." digits] [exponent] | "." digits [exponent] ;
errorlit  = "#REF!" | "#DIV/0!" | "#NAME?" | "#VALUE!" | "#CIRC!" ;
```

- A `ref` whose column > `XFD` or row outside `1..=1048576` → `ErrorLit(Ref)` (shows `#REF!`), never clamped. A token that cannot be read as a reference at all is a syntax error.
- `TRUE`/`FALSE` are matched before references (no `TRUE1`-style ambiguity: letters+digits is a ref, bare letters is a name/boolean).
- Limits: ≤ 8,192 chars, ≤ 256 nesting depth → `ParseError::TooLong` / `TooDeep`.
- Functions in this slice: `SUM`, `AVERAGE`, `MIN`, `MAX`, `COUNT` (≥ 1 arg). Any other name parses and evaluates to `#NAME?`.

## 3. Canonical printing (FR-012)

`"=" + print(expr)` with: upper-case function names and column letters, `,` separator, no spaces, minimal parentheses from binding powers (a right operand of equal precedence keeps its parentheses: `=A1-(B1-C1)`), `$` markers preserved, numbers via `format_full`. Round-trip invariant: `parse(print(e)) == e`.

## 4. Number display (FR-006) — `proto123-value::format_general(n, width)`

Width = 9 for grid cells in this slice. First form that fits:

| Step | Rule | Example (width 9) |
|---|---|---|
| 1 | 15 significant digits, trailing zeros dropped, `-0` → `0` | `=0.1+0.2` → `0.3`; `=10/4` → `2.5` |
| 2 | decimals rounded half-away-from-zero to fit | `=1/3` → `0.3333333`; `0.000001234` → `0.0000012` |
| 3 | scientific `d.dddE±XX`, mantissa digits to fit, trailing zeros dropped — when the integer part does not fit or a nonzero value rounds to 0 | `123456789012` → `1.235E+11`; `0.0000000012` → `1.2E-09` |
| 4 | fill with `#` | (width < scientific minimum) |

The formula bar and live region use `format_full(n)` (step 1 only).
