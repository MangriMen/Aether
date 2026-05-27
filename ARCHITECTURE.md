# Aether Architecture Guidelines

This document defines the code structure, dependency direction, and module boundaries for the project. Adherence to these rules is mandatory for both human developers and AI agents.

---

## 1. Architectural Style: Vertical Slices + Hexagonal (Ports & Adapters)

The codebase is divided into independent functional features (Vertical Slices). Inside each feature, Hexagonal Architecture isolates business logic from external technologies.

### Feature Directory Layout:

- **src/features/auth/**
  - `mod.rs` — Feature Facade (Public Contract)
  - **domain/** — Core Business Logic (Pure Rust only)
    - `mod.rs`, `error.rs`, **model/** (Entities, Value Objects)
  - **app/** — Application Layer (Orchestration)
    - `mod.rs`, `error.rs`, `dtos.rs`, `services.rs`, **use_cases/** (Inbound Ports)
  - **infra/** — Infrastructure Layer (Technical Details)
    - `mod.rs`, **sqlite/** (Outbound Adapter for SQLite)

---

## 2. Dependency Rule

Dependencies must only point **inward**:

**infra ➔ app ➔ domain**

- **domain**: Fully isolated. Zero internal project dependencies. Contains pure logic and invariants.
- **app**: Depends only on `domain`. Implements use cases and communicates with the outside world via **Ports (Traits)**.
- **infra**: Depends on `app` and `domain`. Implements **Adapters** (SQLx, File System, Network, UI clients).

---

## 3. Module Granularity: "File until it gets crowded"

To prevent directory bloat and empty boilerplate `mod.rs` files:

1.  **1–2 entities** must live as a **single file** (`module_name.rs`) at the parent level. Do not create a folder for a single trait or struct.
2.  **3+ entities** (e.g., multiple use cases) justify turning the file into a **directory** containing a `mod.rs` and separate files.

---

## 4. Re-export (pub use) & Boundary Control

Improper re-exports break architectural boundaries and ruin automated linting.

### Rules for the Feature Root (`src/features/[feature_name]/mod.rs`):

1.  **No Layer Wildcards**: `pub use domain::*;` or `pub use infra::*;` at the feature root is strictly prohibited.
2.  **Facade Principle**: Expose **only** what the outside world needs to execute the feature: `Use Cases`, `DTOs`, and `Application Errors`.
3.  **Isolate Infrastructure**: Infrastructure adapters (e.g., `SqliteCredentialsStorage`) must **NOT** be flattened via `pub use`. The Composition Root (`main.rs`) must access them via their full explicit path (e.g., `auth::infra::sqlite::...`). This allows architecture linters to detect illegal infrastructure leaks using the `::infra::` token.

### Internal Re-exports:

Use `pub(crate) use` or `pub(in crate::features::[name]) use` inside layers to keep imports clean without polluting the public workspace API.

---

## 5. Instructions for AI Agents

1.  **Do not bloat the tree**: Add new DTOs or models to existing files unless rule #3 is met.
2.  **Respect Encapsulation**: Never reference `infra` types inside `domain` or `app` (except for implementing ports in `infra`).
3.  **Code Comments**: Keep code comments to an absolute minimum. Use them only for complex algorithmic behavior or compiler workarounds. Code comments must be written strictly in English.
