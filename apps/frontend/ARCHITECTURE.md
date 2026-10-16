# Frontend architecture

The frontend is organised by domain: each feature keeps its UI, state and API calls together.

```
src/
  app/        composition: shell/ (titlebar, sidebar, layouts), runtime/ (bootstrap, init guard,
              global listeners), router, providers, pages that span several domains
  platform/   the Tauri side: bindings/ (generated), ipc/ (commands, events), window/
  features/   one folder per domain: instance, content, plugins, settings, java, ...
  shared/     code that knows nothing about Aether: ui-kit, i18n, formatting, utils
```

## Terms

- **Module**: a folder with a public `index.ts`. Every `features/<name>` is a module, and so
  are `platform/<name>`, `shared/<name>`, `app/shell` and `app/runtime`.
- **Folders with fixed names** inside a feature: `api/`, `model/`, `ui/`, `page/`, `pane/`
  (see below). A feature has no other folders.

## Rules

1. Imports go one way: `app → features → platform, shared`. `platform` and `shared` never
   import `features` or `app`; features never import `app`.
2. Another module is imported only through its `index.ts` (`@/features/instance`,
   `@/platform/ipc`, `@/app/shell`). Inside a module, imports are relative and point at
   files, not at the module's own index. The one exception is `platform/bindings`: it is
   generated, every file exports `commands` and `events`, so it is imported per domain
   (`@/platform/bindings/instance`).
3. Features form no cycles. If two features need each other, move the shared part down
   or let `app/runtime` coordinate them.
4. `invoke` and `listen` are called only in `platform/` and in a feature's `api/`.
5. A page that belongs to one domain lives in that feature's `page/`; `app/router`
   registers routes. A domain's settings pane lives in its `pane/`; `app/shell` only puts
   the panes together.
6. No ports or interfaces until something has a second implementation.

## Inside a feature

A feature is flat. Files lie at its root, next to a few optional folders with fixed names:

| Folder | Holds |
|---|---|
| `index.ts` | the public API: what other modules may use, nothing more |
| `api/` | queries, mutations, query keys, event listeners on top of `platform/ipc` |
| `model/` | types, mappers, stores, helpers |
| `ui/` | components |
| `page/` | the domain's page and the parts only it uses |
| `pane/` | the domain's settings pane |

- No other folders and no nesting: the deepest path is `features/<name>/<folder>/<file>`.
  The only exception is `__tests__/` next to the files it covers.
- A component, its hook and its helpers may also sit side by side at the feature root;
  the folders are there for when a kind of file gets numerous.
- Create a folder only when there is something to put in it.

## Splitting a domain

When a feature grows past ~15–20 files at its root, or a topic inside it collects ~8 files,
that topic becomes a feature of its own, named `<domain>-<part>`:

```
features/instance/            the base: what the parts share
features/instance-create/
features/instance-import/
features/instance-settings/
```

- Parts depend on the base; the base never imports its parts.
- Parts do not import each other. `app` puts them together: the router registers a part's
  page or dialog, `app/shell` places its buttons and panes.
- If two parts need the same thing, it moves into the base.

Example:

```
features/instance/
  index.ts                      export * from './api/instanceQueries'; export { InstanceCard } ...
  api/
    instanceQueries.ts          useInstances, useInstance, instanceKeys
    instanceMutations.ts        useLaunchInstance, useRemoveInstance
    useInstanceEventsListener.ts
  model/
    instance.ts                 Instance type, InstanceState
    mappers.ts                  dto → Instance
    __tests__/mappers.test.ts
  ui/
    InstanceCard.tsx            used by page/ and, through index.ts, by app/pages/home
    InstanceIcon.tsx
    InstancePlayButton.tsx
  page/
    InstancePage.tsx
    InstanceHeaderInfo.tsx

features/instance-settings/
  index.ts                      export { InstanceSettingsDialog } from './InstanceSettingsDialog';
  InstanceSettingsDialog.tsx
  GeneralTab.tsx
  JavaAndMemoryTab.tsx
  useJavaAndMemorySettingsHandler.ts
  defaultInstanceSettingsQueries.ts
  instanceSettingsSchemas.ts

features/java/                  a small feature: no folders at all
  index.ts
  javaQueries.ts
  JavaPane.tsx
  useJavaInstallationListener.ts
```

## Migration

The code is moving off Feature-Sliced Design one domain per PR. Until it is done,
`entities/`, `widgets/`, `pages/` and the old slices in `features/` (listed in
`LEGACY_FEATURES` in `.dependency-cruiser.cjs`) still exist. The checks skip them. When a
slice moves, remove it from that list.

Known exceptions, to be removed when `shared/` is split into modules:
- `shared/` has import cycles through its barrels, so `no-circular` does not cover it yet;
- `shared/model/error.ts` imports error types from `platform/bindings`.

## Checks

- ESLint (`bun lint:code`, also shown in the editor): no deep imports into another module,
  no `@/app` outside `app/`, no feature imports from `shared/` or `platform/`.
- dependency-cruiser (`bun lint:arch`): cycles, layer direction, relative imports across
  features, public API of features and modules. Config: `.dependency-cruiser.cjs`.
- `bun arch:graph` writes a module-level dependency graph to `arch-graph.mmd` (Mermaid;
  paste it into any Mermaid viewer).
