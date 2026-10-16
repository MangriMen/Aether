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
- **Sub-feature**: a folder inside a feature named after a part of the domain (`settings/`,
  `create/`, `versions/`).
- **Reserved folders**: `api/`, `model/`, `ui/`, `page/`, `pane/`. Inside a feature these
  names mean only what is listed below. Any other folder is a sub-feature.

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

| Folder | Holds |
|---|---|
| `index.ts` | the public API: what other modules may use, nothing more |
| `api/` | queries, mutations, query keys, event listeners on top of `platform/ipc` |
| `model/` | types, mappers, stores and helpers used by more than one sub-feature |
| `ui/` | components used by more than one sub-feature |
| `page/` | the domain's page and the parts only it uses |
| `pane/` | the domain's settings pane |
| anything else | a sub-feature |

- Files inside a sub-feature are flat: a component, its hook and its helpers sit side by
  side. Do not split them into `ui/model/lib`.
- A sub-feature with its own requests or types uses the same reserved names inside it
  (`settings/api/`, `settings/model/`).
- When a sub-feature grows past 10–12 files, split it into smaller sub-features.
- Tests live in `__tests__/` next to the files they cover.
- Create a reserved folder only when there is something to put in it.

Example:

```
features/instance/
  index.ts                      export { InstanceCard } from './ui/InstanceCard'; ...
  api/
    instanceQueries.ts          useInstances, useInstance, instanceKeys
    instanceMutations.ts        useLaunchInstance, useRemoveInstance
    useInstanceEventsListener.ts
  model/
    instance.ts                 Instance type, InstanceState
    mappers.ts                  dto → Instance
    __tests__/mappers.test.ts
  ui/
    InstanceCard.tsx            used by page/ and by app/pages/home through index.ts
    InstanceIcon.tsx
    InstancePlayButton.tsx
  page/
    InstancePage.tsx
    InstanceHeaderInfo.tsx
  settings/
    InstanceSettingsDialog.tsx
    GeneralTab.tsx
    JavaAndMemoryTab.tsx
    useJavaAndMemorySettingsHandler.ts
    api/
      defaultInstanceSettingsQueries.ts
    model/
      instanceSettingsSchemas.ts
  create/
    CreateInstanceDialog.tsx
    useCreateInstanceForm.ts
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
