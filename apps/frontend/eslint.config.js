import tsParser from '@typescript-eslint/parser';
import tailwind from 'eslint-plugin-better-tailwindcss';
import perfectionist from 'eslint-plugin-perfectionist';
import solid from 'eslint-plugin-solid/configs/recommended';
import sonarjs from 'eslint-plugin-sonarjs';

// Oxlint (.oxlintrc.json) and Oxfmt (.oxfmtrc.json) own everything they can express natively.
// ESLint keeps only what needs JS plugins, which are slower in oxlint than ESLint with its cache:
// sonarjs, solid, perfectionist/sort-exports, better-tailwindcss,
// padding-line-between-statements and no-restricted-syntax. Unlisted dependencies are caught by knip.

// model/ is plain code: Tauri calls go to api/ (the same text lives in .oxlintrc.json).
const tauriInModelMessage =
  'model/ is plain code: call Tauri from api/ (or from the component or primitive that needs it).';

/** @type {import('eslint').Linter.Config[]} */
export default [
  { files: ['**/*.{js,jsx,ts,tsx}'] },
  {
    ignores: [
      '*.cjs',
      'eslint.config.js',
      'dist',
      '.husky',
      'src-tauri',
      'src/platform/bindings/**/*',
    ],
  },
  {
    languageOptions: {
      parser: tsParser,
      ecmaVersion: 'latest',
      sourceType: 'module',
      parserOptions: { ecmaFeatures: { jsx: true } },
    },
  },
  {
    plugins: { perfectionist },
    rules: { 'perfectionist/sort-exports': ['error'] },
  },
  sonarjs.configs.recommended,
  {
    rules: {
      'sonarjs/aws-restricted-ip-admin-access': 'off',
      'sonarjs/no-unused-vars': 'off',
      'sonarjs/todo-tag': 'off',
      'sonarjs/no-skipped-tests': 'off',
      'sonarjs/no-redundant-optional': 'off',
    },
  },
  tailwind.configs['stylistic-warn'],
  {
    settings: {
      'better-tailwindcss': {
        entryPoint: './src/app/app.css',
        detectComponentClasses: true,
      },
    },
    rules: {
      'better-tailwindcss/enforce-consistent-class-order': 'off',
      'better-tailwindcss/enforce-consistent-line-wrapping': 'off',
    },
  },
  solid,
  {
    rules: {
      'padding-line-between-statements': [
        'error',
        // Empty line before and after multiline blocks (if, switch, for)
        { blankLine: 'always', prev: '*', next: 'multiline-block-like' },
        { blankLine: 'always', prev: 'multiline-block-like', next: '*' },
      ],
    },
  },
  {
    // Oxlint's no-restricted-imports sees only static imports and import('literal');
    // the template-literal form of a dynamic import is checked here.
    files: ['src/features/*/model/**/*.{ts,tsx}'],
    ignores: ['**/__tests__/**'],
    rules: {
      'no-restricted-syntax': [
        'error',
        {
          selector:
            'ImportExpression[source.quasis.0.value.raw=/^@tauri-apps/]',
          message: tauriInModelMessage,
        },
      ],
    },
  },
];
