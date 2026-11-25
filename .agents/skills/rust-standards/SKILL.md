---
name: rust-standards
description: Gold standard software engineering practices in Rust for production-grade code. Focuses on memory safety, strict typing, explicit error handling, data-race-free concurrency, predictable performance, and rigorous automated verification.
---

# Rust Systems Engineering Skill

This skill defines the mandatory architectural, idiomatic, and operational standards for writing, refactoring, auditing, and testing production-grade Rust code.

## 1. Core Philosophy and Invariants

1. **Compile-Time Correctness:** Leverage the type system to make illegal states unrepresentable (*Make Illegal States Unrepresentable*).
2. **Zero Panics in Production:** Library or service code must never `panic!` during normal execution paths or due to invalid external inputs.
3. **Zero Unjustified `unsafe`:** The use of `unsafe` is strictly prohibited unless unavoidable for FFI or benchmark-proven optimizations, and requires documented safety invariants (`// SAFETY: ...`).
4. **Zero-Cost and Predictability:** Prefer zero-cost abstractions (iterators, generics with static dispatch) and avoid unnecessary allocations on the *hot path*.

## 2. Code Standards and Idioms

### 2.1. Error Handling

* **Strict Prohibitions:** Never use `.unwrap()` or `.expect()` in production code. They are permitted *only* in tests (`#[cfg(test)]`) or when an absolute mathematical invariant guarantees failure is impossible (which must be justified with a comment).
* **Libraries (`lib.rs`):** Use `thiserror` to define strongly typed, exhaustive, and semantically rich error enums. Never leak internal dependency error types into the public API without encapsulation.
* **Applications (`main.rs` / binaries):** Use `anyhow` or `eyre` for error propagation with rich context (`.context("failed to load configuration")?`).
* **Propagation:** Always use the `?` operator with clean conversions via `From` / `Into`.

### 2.2. Type System and API Design

* **Newtype Pattern:** Wrap primitive types that carry domain meaning (e.g., `struct UserId(Uuid);`, `struct Email(String);`) to prevent accidental parameter swapping.
* **Typestate Pattern:** For state machines or complex builders, encode states as generic type parameters to enforce valid transitions at compile time.
* **Flexible Signatures:**
  * Accept `&str` instead of `&String`.
  * Accept `&[T]` instead of `&Vec<T>`.
  * Accept `impl AsRef<Path>` for file paths.
  * Accept `impl Into<String>` when the function needs to take ownership of a string.
* **Standard Derives:** Every public type should derive, whenever semantically valid: `#[derive(Debug, Clone, PartialEq, Eq)]`. Add `Hash`, `Default`, `Serialize`, `Deserialize` as needed.
* **Visibility:** Restrict visibility to the minimum required (`pub(crate)`, `pub(super)`). Expose only the minimal surface area demanded by the public API.

### 2.3. Memory Management and Ownership

* Avoid impulsive `.clone()` calls just to satisfy the *borrow checker*. Analyze data lifecycles and deliberately use references (`&T`, `&mut T`), `Cow<'a, T>` (Copy-on-Write), or smart pointers (`Arc<T>`, `Rc<T>`).
* Prefer iterators and combinators (`.iter()`, `.map()`, `.filter()`, `.fold()`) over index-based loops (`for i in 0..vec.len()`), eliminating bounds checks and preventing off-by-one errors.

### 2.4. Asynchronous Programming and Concurrency

* **Runtime:** Use `tokio` as the default runtime for asynchronous I/O.
* **Never Block the Executor:** Never execute CPU-intensive synchronous operations, blocking file I/O (`std::fs`), or `std::thread::sleep` inside an `async fn`. Use `tokio::task::spawn_blocking` or `tokio::fs`.
* **Synchronization:**
  * Use `std::sync::Mutex` (or `parking_lot::Mutex`) for short-lived locks that **do not** cross `.await` points.
  * Use `tokio::sync::Mutex` only when the lock must be held across an `.await`.
* **Cancel Safety:** When using `tokio::select!`, ensure all involved futures are *cancel-safe* or explicitly document the behavior in the event of partial drops.

## 3. Documentation and Observability

* **Mandatory Docstrings:** Every `pub` item (modules, structs, enums, traits, functions) must have `///` documentation explaining:
  * What it does and why.
  * `# Errors`: Conditions under which it returns `Err`.
  * `# Panics`: Panic conditions (ideally none).
  * `# Examples`: A working example that compiles as a doctest.
* **Structured Logging:** Use the `tracing` crate (`info!`, `warn!`, `error!`, `debug!`, `#[instrument]`) instead of `println!` or `log`. Always include structured fields (e.g., `tracing::info!(user_id = %id, "user authenticated")`).

## 4. Task Execution Workflow (Agent Workflow)

When receiving any implementation, bug-fixing, or refactoring task, strictly follow these steps in order:

### Step 1: Architecture and Dependency Analysis

1. Inspect `../../../Cargo.toml` and the directory structure before writing any code.
2. Check existing crate versions and maintain consistency with the project's ecosystem.
3. Plan data types and function signatures before implementation.

### Step 2: Incremental Implementation

1. Define domain models and error types first.
2. Implement pure business logic decoupled from the I/O layer (infrastructure/network/database) to facilitate deterministic unit testing.
3. Write clean, modular code with cohesive functions (maximum ~40-50 lines per function).

### Step 3: Automated Verification (Quality Gate)

No task is considered complete until all commands below pass without errors or warnings:

```bash
# 1. Strict formatting
cargo fmt --all -- --check

# 2. Rigorous linter (treats warnings as errors and enables pedantic lints)
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -A clippy::module_name_repetitions -A clippy::missing_errors_doc

# 3. Full compilation check (including examples and benchmarks)
cargo check --all-targets --all-features

# 4. Execution of all unit, integration, and doc tests
cargo test --all-features
```

*(Note: If `clippy::pedantic` triggers specific false positives within the project's context, justify and suppress them locally with `#[allow(clippy::lint_name)]` accompanied by an explanatory comment—never globally without reason).*

## 5. Testing Strategy

* **Unit Tests:** Must reside in the same file within `#[cfg(test)] mod tests { use super::*; ... }`.
* **Naming Convention:** Use descriptive names in the format `test_<functionality>_<scenario>_<expected_outcome>` (e.g., `test_parse_token_expired_returns_unauthorized_error`).
* **Edge-Case Coverage:** Mandatory testing for:
  * Happy paths.
  * Empty, null, or boundary inputs (`0`, `usize::MAX`, empty strings, multibyte Unicode characters).
  * Simulated dependency failures (via traits and mocks/stubs).
* **Property-Based Testing:** For parsers, serializers, and complex algorithms, consider using `proptest` or `quickcheck`.

## 6. Final Review Checklist (Self-Review)

Before delivering the final response to the user, audit the generated code against this checklist:

* [ ] Is there any `.unwrap()` or `.expect()` outside of test modules?
* [ ] Is there any forgotten `println!`, `dbg!`, or `todo!()` in the code?
* [ ] Do all public types derive `Debug` and have `///` documentation?
* [ ] Are allocations (`.clone()`, `String`, `Vec`) genuinely necessary, or can they be replaced with references/slices?
* [ ] Is the asynchronous code free of blocking calls?
* [ ] Is `../../../Cargo.toml` clean, free of unused dependencies, and configured with minimal features (`default-features = false` where appropriate)?
* [ ] Have all *Quality Gate* commands (`fmt`, `clippy`, `test`) been executed and passed successfully?