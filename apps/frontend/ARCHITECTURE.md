# Frontend architecture

The frontend is organised by domain, like the backend (`aether-core/src/features/*`).
The layout is close to the one the Modrinth App adopted (`app / platform / features / shared`).

```
src/
  app/        composition: shell/ (titlebar, sidebar, layouts), runtime/ (bootstrap, init guard,
              global listeners), router, providers, pages that span several domains
  platform/   the Tauri side: bindings/ (generated), ipc/ (commands, events), window/
  features/   one folder per domain: instance, content, plugins, settings, java, ...
  shared/     code that knows nothing about Aether: ui-kit, i18n, formatting, utils
```

## Rules

1. Imports go one way: `app → features → platform, shared`. `platform` and `shared` never
   import `features` or `app`; features never import `app`.
2. A feature is used from the outside only through `features/<name>/index.ts`. Inside a
   feature, imports are relative; across features, they use the `@/features/<name>` alias.
3. Features form no cycles. If two features need each other, move the shared part down
   or let `app/runtime` coordinate them.
4. Inside a feature: `api/` (queries, mutations, keys on top of `platform`) and `model/` at
   the root for shared code; everything else is grouped by sub-feature (`versions/`,
   `filters/`, `settings/`) with flat files, no `ui/model/lib` split.
5. `invoke` and `listen` are called only in `platform/` and `features/*/api`.
6. A page that belongs to one domain lives in that feature; `app/router` registers routes.
   A settings pane lives in its domain too; `app/shell` only puts the panes together.
7. No ports or interfaces until something has a second implementation.

## Migration

The code is moving off Feature-Sliced Design one domain per PR. Until it is done,
`entities/`, `widgets/`, `pages/` and the old slices in `features/` (listed in
`LEGACY_FEATURES` in `.dependency-cruiser.cjs`) still exist. The checks skip them. When a
slice moves, remove it from that list.

Known exceptions, to be removed when `shared/` is split into modules:
- `shared/` has import cycles through its barrels, so `no-circular` does not cover it yet;
- `shared/model/error.ts` imports error types from `platform/bindings`.

## Checks

- ESLint (`bun lint:code`, also shown in the editor): no deep `@/features/<name>/...`
  imports, no `@/app` outside `app/`, no feature imports from `shared/` or `platform/`.
- dependency-cruiser (`bun lint:arch`): cycles, layer direction, relative imports across
  features, feature public API. Config: `.dependency-cruiser.cjs`.
- `bun arch:graph` writes a module-level dependency graph to `arch-graph.mmd` (Mermaid;
  paste it into any Mermaid viewer).
