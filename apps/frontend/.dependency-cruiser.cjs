// Architecture boundaries, see ARCHITECTURE.md. The rules cover every file under src/.

const FEATURE = '^src/features/([^/]+)/';
const FEATURE_INDEX = `${FEATURE}index\\.tsx?$`;
// Modules outside features/. platform/bindings is generated and has no index:
// every file exports `commands` and `events`, so it is imported per domain.
const MODULE = '^src/(app/(shell|runtime|pages/[^/]+)|platform/(?!bindings/)[^/]+|shared/[^/]+)/';
const MODULE_INDEX = '^src/(app/(shell|runtime|pages/[^/]+)|platform/[^/]+|shared/[^/]+)/index\\.tsx?$';
// The composition root: files at the top of app/ (App, AppRoot, AppRouter, route patterns).
const APP_ROOT = '^src/app/[^/]+$';
// Anything in src/ that is not part of the layout (see src-layout).
const OUTSIDE_LAYOUT = [
  '^src/(?!index\\.tsx$|app/|platform/[^/]+/|features/[^/]+/|shared/[^/]+/)',
  '^src/app/(?![^/]+$|shell/|runtime/|pages/[^/]+/)',
];

/** @type {import('dependency-cruiser').IConfiguration} */
module.exports = {
  forbidden: [
    {
      name: 'no-circular',
      comment:
        'No import cycles between files (type-only imports do not count; scripts/check-module-cycles.ts sees them between modules).',
      severity: 'error',
      from: { path: '^src/' },
      to: { circular: true, viaOnly: { dependencyTypesNot: ['type-only'] } },
    },
    {
      name: 'src-layout',
      comment:
        'src/ holds index.tsx, app/ (shell/, runtime/, pages/<page>/ and the composition root), platform/<module>/, features/<feature>/ and shared/<module>/; nothing else.',
      severity: 'error',
      from: { path: OUTSIDE_LAYOUT },
      to: {},
    },
    {
      name: 'src-layout-target',
      comment:
        'The same layout, seen from the importing side: catches a misplaced file that imports nothing. A file that imports nothing and is imported by nothing is left to knip.',
      severity: 'error',
      from: {},
      to: { path: OUTSIDE_LAYOUT },
    },
    {
      name: 'app-modules-not-root',
      comment: 'app/shell, app/runtime and app/pages do not import the composition root at the top of app/.',
      severity: 'error',
      from: { path: '^src/app/[^/]+/' },
      to: { path: APP_ROOT },
    },
    {
      name: 'platform-is-low-level',
      comment: 'platform/ does not import app or features.',
      severity: 'error',
      from: { path: '^src/platform/' },
      to: { path: '^src/(app|features)/' },
    },
    {
      name: 'shared-is-low-level',
      comment: 'shared/ does not import app or features.',
      severity: 'error',
      from: { path: '^src/shared/' },
      to: { path: '^src/(app|features)/' },
    },
    {
      name: 'shared-knows-no-platform',
      comment: 'shared/ is not tied to Aether, so it does not import platform/.',
      severity: 'error',
      from: { path: '^src/shared/' },
      to: { path: '^src/platform/' },
    },
    {
      name: 'ui-kit-only-utils',
      comment:
        'shared/ui-kit imports no shared module but shared/utils; its texts come from the app through UiKitLabelsProvider.',
      severity: 'error',
      from: { path: '^src/shared/ui-kit/' },
      to: { path: '^src/shared/(?!ui-kit/|utils/)' },
    },
    {
      name: 'feature-not-app',
      comment: 'Features do not import app/.',
      severity: 'error',
      from: { path: FEATURE },
      to: { path: '^src/app/' },
    },
    {
      name: 'feature-no-relative-cross',
      comment: 'Another feature is imported as @/features/<name>, never through a relative path.',
      severity: 'error',
      from: { path: FEATURE },
      to: { path: '^src/features/', pathNot: '^src/features/$1/', dependencyTypesNot: ['aliased'] },
    },
    {
      name: 'feature-public-api',
      comment: 'Outside code reaches a feature only through its index.ts.',
      severity: 'error',
      from: { pathNot: '^src/features/' },
      to: { path: FEATURE, pathNot: FEATURE_INDEX },
    },
    {
      name: 'feature-public-api-cross',
      comment: 'Another feature is reached only through its index.ts.',
      severity: 'error',
      from: { path: '^src/features/([^/]+)/' },
      to: { path: FEATURE, pathNot: ['^src/features/$1/', FEATURE_INDEX] },
    },
    {
      name: 'feature-base-not-parts',
      comment:
        'A domain split into features (plugins, plugins-settings, ...) depends one way: the base never imports its parts. Parts may import each other without cycles.',
      severity: 'error',
      from: { path: '^src/features/([^/-]+)/' },
      to: { path: '^src/features/$1-[^/]+/' },
    },
    {
      name: 'feature-shape',
      comment:
        'A feature holds index.ts at its root and everything else in api/, model/, lib/ or ui/, none of them nested (__tests__/ aside). Split a big domain into <domain>-<part> features instead.',
      severity: 'error',
      from: {
        path: [
          `${FEATURE}(?!index\\.tsx?$)[^/]+$`,
          `${FEATURE}(?!(?:api|model|lib|ui|__tests__)/)[^/]+/`,
          `${FEATURE}(?:api|model|lib|ui)/(?!__tests__/)[^/]+/`,
        ],
      },
      to: {},
    },
    {
      name: 'feature-file-names',
      comment:
        'ui/ holds PascalCase .tsx components, model/ holds no .tsx, lib/ holds only use*/create* primitives.',
      severity: 'error',
      from: {
        path: [
          `${FEATURE}ui/(?:[^A-Z][^/]*|[^/]+\\.ts)$`,
          `${FEATURE}model/[^/]+\\.tsx$`,
          `${FEATURE}lib/(?!(?:use|create)[A-Z])[^/]+$`,
        ],
      },
      to: {},
    },
    {
      name: 'shared-shape',
      comment:
        'A shared module is flat: index.ts and its files (__tests__/ aside). ui-kit alone has folders: primitives/, composite/, lib/, model/, nothing at its root but index.ts.',
      severity: 'error',
      from: {
        path: [
          '^src/shared/(?!ui-kit/)[^/]+/(?!__tests__/)[^/]+/',
          '^src/shared/ui-kit/(?!index\\.tsx?$)[^/]+$',
          '^src/shared/ui-kit/(?!(?:primitives|composite|lib|model|__tests__)/)[^/]+/',
          '^src/shared/ui-kit/(?:primitives|composite|lib|model)/(?!__tests__/)[^/]+/',
        ],
      },
      to: {},
    },
    {
      name: 'shared-file-names',
      comment:
        'In shared/, .tsx files are PascalCase components and .ts files are camelCase. In ui-kit, primitives/ and composite/ hold components, lib/ only use*/create*, model/ no .tsx.',
      severity: 'error',
      from: {
        path: [
          '^src/shared/[^/]+/[^A-Z/][^/]*\\.tsx$',
          '^src/shared/ui-kit/[^/]+/[^A-Z/][^/]*\\.tsx$',
          '^src/shared/[^/]+/[A-Z][^/]*\\.ts$',
          '^src/shared/ui-kit/[^/]+/[A-Z][^/]*\\.ts$',
          '^src/shared/ui-kit/(?:primitives|composite)/[^/]+\\.ts$',
          '^src/shared/ui-kit/lib/(?!(?:use|create)[A-Z])[^/]+$',
          '^src/shared/ui-kit/model/[^/]+\\.tsx$',
        ],
      },
      to: {},
    },
    {
      name: 'module-public-api',
      comment: 'Other modules are reached only through their index.ts; inside a module, imports are relative.',
      severity: 'error',
      // $1 is the importing module (first two path segments)
      from: { path: '^src/([^/]+/[^/]+)' },
      to: { path: MODULE, pathNot: ['^src/$1/', MODULE_INDEX] },
    },
  ],
  options: {
    doNotFollow: { path: 'node_modules' },
    tsPreCompilationDeps: true,
    tsConfig: { fileName: 'tsconfig.json' },
    enhancedResolveOptions: {
      exportsFields: ['exports'],
      conditionNames: ['import', 'require', 'node', 'default', 'types'],
      extensions: ['.ts', '.tsx', '.js', '.jsx', '.d.ts'],
    },
    skipAnalysisNotInRules: true,
    cache: { folder: 'node_modules/.cache/dependency-cruiser', strategy: 'content' },
  },
};
