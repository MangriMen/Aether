// Architecture boundaries, see ARCHITECTURE.md.
// While the code moves off the old FSD layers, the rules only cover the new layout:
// entities/, widgets/, pages/ and the old slices in features/ are skipped.
// Drop a slice from LEGACY_FEATURES once it has moved.

const LEGACY_FEATURES = [
  'install-plugin-from-github',
  'instance-action-button',
  'instance-settings',
  'plugin-controlled-card',
  'plugin-settings-form',
  'restart-plugin-button',
  'toggle-plugin-button',
  'update-badge',
];

const FEATURE = `^src/features/(?!(?:${LEGACY_FEATURES.join('|')})/)([^/]+)/`;
const FEATURE_INDEX = `${FEATURE}index\\.tsx?$`;
const OLD_LAYERS = '^src/(entities|widgets|pages)/';

/** @type {import('dependency-cruiser').IConfiguration} */
module.exports = {
  forbidden: [
    {
      name: 'no-circular',
      comment: 'New code takes no part in import cycles (type-only imports do not count).',
      severity: 'error',
      from: { path: ['^src/platform/', '^src/app/(shell|runtime)/', FEATURE] },
      to: { circular: true, viaOnly: { dependencyTypesNot: ['type-only'] } },
    },
    {
      name: 'platform-is-low-level',
      comment: 'platform/ does not import app or features.',
      severity: 'error',
      from: { path: '^src/platform/' },
      to: { path: ['^src/(app|features)/', OLD_LAYERS] },
    },
    {
      name: 'shared-is-low-level',
      comment: 'shared/ does not import app or features.',
      severity: 'error',
      from: { path: '^src/shared/' },
      to: { path: ['^src/(app|features)/', OLD_LAYERS] },
    },
    {
      name: 'shared-knows-no-platform',
      comment:
        'shared/ is not tied to Aether. Exceptions: the error types and the test that checks error translations against the bindings.',
      severity: 'error',
      from: { path: '^src/shared/', pathNot: ['^src/shared/model/error\\.ts$', '^src/shared/i18n/__tests__/'] },
      to: { path: '^src/platform/' },
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
