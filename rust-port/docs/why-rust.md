# Why Rust?

Choose `seleniumbase-rs` when the test harness itself benefits from strong
compile-time checks, predictable resource ownership, native binaries, and
integration with a Rust application or workspace.

## Practical advantages

| Area | Rust advantage | Important limit |
|------|----------------|-----------------|
| Refactoring | Types and compiler errors expose many broken call sites before a test runs | Selectors and page behavior still fail at runtime |
| Concurrency | Tokio tasks and explicit shared state support controlled parallelism | Browsers remain expensive external processes |
| Cleanup | Ownership makes lifecycle boundaries visible | Async browser cleanup cannot run from `Drop` |
| Distribution | `cargo install` produces a native CLI without a Python or Node runtime | Browser and driver installation are still required |
| Dependencies | Cargo lockfiles and feature flags provide reproducible dependency selection | Dependencies still require auditing and updates |
| Integration | Tests can share typed models and helpers with Rust services | Cross-language teams may prefer their existing stack |

## What you gain over Python

Python SeleniumBase is the upstream project with the largest existing example
library, but a Rust harness removes whole categories of maintenance problems:

- **Type safety**: Renaming a method, changing an assertion signature, or
  mismatching a selector helper is caught at compile time instead of during a
  long CI run.
- **Feature flags**: Optional cloud integrations, Playwright, and the MCP
  server are compiled in only when you enable them, keeping default builds
  small.
- **Native distribution**: The `sbase` CLI is a single binary; tests do not
  require managing a Python virtual environment or resolving pip conflicts.
- **Shared codebase**: Rust services and their browser tests can share structs,
  serializers, and validation logic from the same crate.

## What you gain over JavaScript / TypeScript

JavaScript tooling is convenient for frontend teams, yet Rust offers
meaningful engineering guarantees:

- **Compile-time correctness**: Missing `await`, stale imports, and renamed
  APIs surface before any browser launches.
- **Deterministic resource cleanup**: The `run_browser_test` helper awaits the
  test body and then quits the WebDriver session, so cleanup failures are
  reported instead of silently dropped.
- **Smaller runtime footprint**: The compiled CLI starts quickly and does not
  pull in a full Node runtime or browser-polyfill stack.

## Concrete example: catching a refactor bug at compile time

Imagine renaming a helper from `login_as` to `sign_in_as`. In Python or
JavaScript every call site is a runtime lottery. In Rust the compiler lists
every broken test immediately:

```text
error[E0599]: no method named `login_as` found for struct `BaseCase`
  --> tests/auth.rs:12:9
   |
12 |         sb.login_as("alice").await?;
   |         ^^^^^^^^ method not found in `BaseCase`
```

The browser still validates the page, but the harness no longer hides trivial
structural mistakes.

## When Python or JavaScript may fit better

Python SeleniumBase has the mature upstream ecosystem and a large collection
of existing tests. JavaScript and TypeScript can be a natural fit for teams
already using browser-first tooling. Migration has a cost, so choose Rust for
specific engineering benefits rather than assuming that one language makes
every browser test faster or safer.

Read the focused discussions on [security](security.md),
[reliability](reliability.md), and [performance](performance.md).

