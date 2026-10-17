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
- **Folders with fixed names** inside a feature: `api/`, `model/`, `lib/`, `ui/` (see
  below). A feature has no other folders.

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
5. A feature exports its entry points (a page, a dialog, a settings pane) through
   `index.ts`; `app` decides where they appear (routes in `app/router`, screens in
   `app/shell`). A page that combines several domains lives in `app/pages`.
6. No ports or interfaces until something has a second implementation.

## Inside a feature

A feature has the same shape everywhere: `index.ts` at the root, every other file in one of
the folders below. The folder tells what kind of file it is.

| Folder | Holds |
|---|---|
| `index.ts` | the public API: what other modules may use, nothing more |
| `api/` | everything that talks to the backend: queries, mutations, query keys, event listeners on top of `platform/ipc`, even when named `use*` |
| `model/` | plain TypeScript without JSX: types, constants, pure functions, mappers, stores and contexts |
| `lib/` | Solid primitives that serve components (`use*`, `create*`): form handlers, table data, actions; nothing that talks to the backend |
| `ui/` | components only, including the feature's pages, dialogs and panes |

- No other folders, no files at the root besides `index.ts`, no nesting: the path is always
  `features/<name>/<folder>/<file>`. The only exception is `__tests__/` next to the files it
  covers.
- Which folder, by file:
  - calls `platform/ipc` → `api/`;
  - `.tsx` with a component, file name in PascalCase → `ui/`;
  - starts with `use` or `create` and is neither of the above → `lib/`;
  - anything else (types, pure functions, mappers, stores) → `model/`.
- `lib/` holds nothing else: a helper without reactivity goes to `model/`, a component to
  `ui/`. The check enforces the file names (see below).
- The role of a component shows in its name, not in a folder: `InstancePage.tsx`,
  `InstanceSettingsDialog.tsx`, `JavaPane.tsx`.
- Create a folder only when there is something to put in it; a small feature may have
  just `api/` and `ui/`.

## Splitting a domain

When one folder of a feature grows past ~15 files, or a topic inside it collects ~8 files,
that topic becomes a feature of its own, named `<domain>-<part>`:

```
features/instance/            the base: what the parts share
features/instance-settings/   launch settings: schemas, fields, forms
features/instance-defaults/   default launch settings and their pane
features/instance-edit/       the settings dialog of one instance
features/instance-create/
features/instance-import/
features/instance-page/
```

- Parts depend on the base; the base never imports its parts.
- Parts may import each other, one way only: no cycles. `app` places their entry points.
- If two parts need the same thing, it moves into the base.
- The size limits are a guide: split when it makes the code easier to follow.

In the code: `plugins` is the base (queries, models, cards, actions), `plugins-settings` is
the settings tab of one plugin, `plugins-manager` is the settings pane with the list,
details and install views; it shows the tab from `plugins-settings`. The settings dialog
in `instance-edit` and the pane in `instance-defaults` both use the forms from
`instance-settings`. `InstancePage` is a part of its own because it shows the content of
an instance, and content depends on the base.

Example:

```
features/instance/
  index.ts                      export * from './api/instanceQueries'; ...
  api/
    instanceQueries.ts          useInstances, useInstance, useEditInstance
    instanceLaunch.ts           useLaunchInstance, useStopInstance
    instanceQueryKeys.ts
    useInstanceEventsListener.ts
  model/
    instance.ts                 Instance type
    instanceMappers.ts          dto → Instance
    formatTimePlayed.ts
  lib/
    useInstanceActions.tsx      launch/stop/remove with confirmations, used by several components
  ui/
    InstanceCard.tsx            used through index.ts by app/pages/home
    InstanceHeaderInfo.tsx
    InstanceIcon.tsx
    InstancePlayButton.tsx

features/instance-edit/
  index.ts                      export * from './ui/InstanceSettingsDialog';
  model/
    instanceSettingsConverter.ts
  lib/
    useJavaAndMemorySettingsHandler.ts
  ui/
    InstanceSettingsDialog.tsx  exported through index.ts, the route is in app/router
    GeneralTab.tsx
    JavaAndMemoryTab.tsx        renders JavaAndMemorySettingsForm from instance-settings

features/java/                  a small feature: only the folders it needs
  index.ts
  api/
    javaQueries.ts
    useJavaInstallationListener.ts
  ui/
    JavaPane.tsx                exported through index.ts, placed on the settings screen by app/shell
```

## Migration

The code is moving off Feature-Sliced Design one domain per PR. Until it is done,
`entities/`, `widgets/` and `pages/` still exist. The checks skip them.

Known exceptions, to be removed when `shared/` is split into modules:
- `shared/` has import cycles through its barrels, so `no-circular` does not cover it yet;
- `shared/model/error.ts` imports error types from `platform/bindings`.

## Checks

- ESLint (`bun lint:code`, also shown in the editor): no deep imports into another module,
  no `@/app` outside `app/`, no feature imports from `shared/` or `platform/`.
- dependency-cruiser (`bun lint:arch`): cycles, layer direction, relative imports across
  features, public API of features and modules, a domain base importing its parts, feature
  folders and file names. Config: `.dependency-cruiser.cjs`. Its cycle check works on files,
  so `scripts/check-feature-cycles.ts` (part of `lint:arch`) also checks the graph of features,
  type-only imports included.
- `bun arch:graph` writes a module-level dependency graph to `arch-graph.mmd` (Mermaid;
  paste it into any Mermaid viewer).
