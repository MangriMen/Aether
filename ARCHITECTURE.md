# Aether Architecture Guidelines

This document defines the code structure, dependency direction, and module boundaries for the project. Adherence to these rules is mandatory for both human developers and AI agents.

---

## 1. Architectural Style: Vertical Slices + Hexagonal (Ports & Adapters)

The codebase is divided into independent functional features (Vertical Slices). Inside each feature, Hexagonal Architecture isolates business logic from external technical frameworks and delivery mechanisms.

### Feature Directory Layout:

- **src/features/[feature_name]/**
  - `mod.rs` — Feature Facade (Public Contract)
  - **domain/** — Core Business Logic (Pure language constructs only)
    - Entities, Value Objects, and Domain invariants.
  - **app/** — Application Layer (Orchestration and Use Cases)
    - Inbound Ports, Data Transfer Objects (DTOs), and internal application services.
  - **infra/** — Infrastructure Layer (Technical Details and Outbound Adapters)
    - Low-level implementations (Storage, Network, File System, OS APIs).

---

## 2. Dependency Rule

Dependencies must only point **inward**:

**infra ➔ app ➔ domain**

- **domain**: Fully isolated. Zero internal project dependencies. Contains pure logic and business rules.
- **app**: Depends only on `domain`. Implements use cases and communicates with the outside world exclusively via **Ports (Abstractions/Interfaces)**.
- **infra**: Depends on `app` and `domain`. Implements **Adapters** by binding specific third-party libraries and external systems to the ports defined in the application layer.

---

## 3. Module Granularity: "File until it gets crowded"

To prevent directory bloat, deep nesting, and redundant transit module files across all layers (`domain`, `app`, `infra`), the project strictly adheres to a uniform structural rule:

1. **Low Density (1–2 Entities)**: Must live as a **single file** (`module_name.rs`) directly under the layer root. Creating a subdirectory with a separate module entry point for 1–2 files is strictly prohibited.
   - _Exception_: Rich Domain Models or adapters with heavy internal business logic (e.g., ~150+ lines of validation/invariants) must remain isolated in separate files within a directory to preserve readability. Granularity should be measured by logical density, not raw file counts.
2. **High Density (3+ Entities)**: When a specific sub-layer grows to 3 or more tightly coupled files (e.g., numerous separate use cases or multi-file adapter implementations), it justifies turning that file into a **directory** containing a local module entry point (`mod.rs`) to manage the internal files.

---

## 4. Re-export (pub use) & Boundary Control

Improper re-exports break architectural boundaries and ruin automated linting.

### Rules for the Feature Root (`src/features/[feature_name]/mod.rs`):

1. **No Layer Wildcards**: Broad layer re-exports (e.g., `pub use domain::*;` or `pub use infra::*;`) at the feature root are strictly prohibited.
2. **Facade Principle**: The feature root acts as a strict facade. It must expose **only** what the outside world requires to interact with the slice: `Use Cases` (Inbound Ports), `DTOs`, and `Application Errors`.
3. **Pragmatic Model Exposure**: Pure data-container domain models (Anemic Models / Value Objects without private invariants or security risks) **may** be re-exported from the feature root via explicit `pub use` for direct use by other Rust features.
4. **No Infra Leaks**: Infrastructure types must **NOT** be re-exported from the feature root. This ensures every import of an infra type contains the `::infra::` token.

### Rules for the Infra Layer (`src/features/[feature_name]/infra/mod.rs`):

1. **Submodules are internal**: Group adapters into submodules (e.g., `sqlite/`, `file_system/`) as needed, but keep them `mod` (private) — not `pub mod`.
2. **Re-export is required**: Use explicit re-exports (e.g., `pub use sqlite::SqliteCredentialsStorage;`) so callers access adapters via `[feature_name]::infra::SqliteCredentialsStorage`, not `[feature_name]::infra::sqlite::SqliteCredentialsStorage`.
3. **Rationale**: The import path retains the `::infra::` token for linters, while hiding the internal module structure from consumers.

### Internal Re-exports:

Use internal visibility modifiers (e.g., `pub(crate) use` or module-scoped visibility) inside layers to maintain clean syntax within the feature boundaries without polluting the public workspace API.

---

## 5. Instructions for AI Agents

1. **Do not bloat the tree**: Treat code encapsulation as high priority. Add new DTOs, models, or adapters to existing files unless the density threshold or complexity exception in Rule #3 is met.
2. **Respect Encapsulation**: Never reference `infra` types inside `domain` or `app` (except when implementing ports inside the `infra` layer itself). Cross-feature communication must happen exclusively by invoking another feature's public Use Cases or accessing explicitly exposed safe domain models.
3. **UI Boundary DTOs**: DTO conversion for presentation layers (e.g., Tauri Commands delivering data to the frontend) must live at the boundary interface (the Tauri layer itself), separating presentation serializability from internal Rust backend communication.
4. **Code Comments**: Keep code comments to an absolute minimum. Use them only for complex algorithmic behavior or compiler workarounds. Code comments must be written strictly in English.
