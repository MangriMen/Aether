import globals from 'globals';
import js from '@eslint/js';
import ts from 'typescript-eslint';
import eslintPluginPrettierRecommended from 'eslint-plugin-prettier/recommended';
import tsParser from '@typescript-eslint/parser';
import sonarjs from 'eslint-plugin-sonarjs';
import jsxA11y from 'eslint-plugin-jsx-a11y';
import tailwind from 'eslint-plugin-better-tailwindcss';
import solid from 'eslint-plugin-solid/configs/recommended';
import perfectionist from 'eslint-plugin-perfectionist';
import react from 'eslint-plugin-react';
import importX from 'eslint-plugin-import-x';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Architecture boundaries (see ARCHITECTURE.md); dependency-cruiser checks the rest.
const deepFeatureImport = {
  regex: '^@/features/[^/]+/.+',
  message: 'Import a feature through its index: @/features/<name>.',
};
const deepModuleImport = {
  regex: '^@/(app/(shell|runtime|pages/[^/]+)|platform/(?!bindings/)[^/]+|shared/[^/]+)/.+',
  message: 'Import a module through its index; inside a module, use relative imports.',
};
const appImport = {
  regex: '^@/app(/|$)',
  message: 'Only app/ imports app/.',
};
const upperLayerImport = {
  regex: '^@/features(/|$)',
  message: 'shared/ and platform/ do not import features.',
};
const platformImport = {
  regex: '^@/platform(/|$)',
  message: 'shared/ is not tied to Aether, so it does not import platform/.',
};
const uiKitSharedImport = {
  regex: '^@/shared/(?!utils$)',
  message: 'shared/ui-kit imports only shared/utils; texts come through UiKitLabelsProvider.',
};
// model/ is plain code: Tauri calls (plugins, window, ...) go to api/ or to the code that uses them.
const tauriInModel = {
  group: ['@tauri-apps/**'],
  allowTypeImports: true,
  message: 'model/ is plain code: call Tauri from api/ (or from the component or primitive that needs it).',
};
const featurePatterns = [deepFeatureImport, deepModuleImport, appImport];
const platformPatterns = [...featurePatterns, upperLayerImport];
const sharedPatterns = [...platformPatterns, platformImport];
const uiKitPatterns = [...sharedPatterns, uiKitSharedImport];

/** @type {import('eslint').Linter.Config[]} */
export default [
  { files: ['**/*.{js,jsx,ts,tsx}'] },
  {
    ignores: ['*.cjs', 'eslint.config.js', 'dist', '.husky', 'src-tauri', "src/platform/bindings/**/*"],
  },
  {
    languageOptions: {
      parser: tsParser,
      ecmaVersion: 'latest',
      sourceType: 'module',
      globals: globals.browser,
      parserOptions: {
        ecmaFeatures: {
          jsx: true,
        },
      },
    },
  },
  {
    rules: {
      'no-unused-vars': 'off',
      'prefer-object-spread': 'warn',
      'object-shorthand': ['error', 'always'],
      'no-debugger': 'warn',
      'no-console': 'warn',
    },
  },
  js.configs.recommended,
  ...ts.configs.recommended,
  {
    plugins: { 'import-x': importX },
    rules: {
      'import-x/no-extraneous-dependencies': [
        'error',
        {
          includeTypes: true,
          packageDir: [__dirname, path.resolve(__dirname, '../..')],
        },
      ],
    },
  },
  {
    rules: {
      '@typescript-eslint/no-unused-vars': [
        'error',
        {
          argsIgnorePattern: '^_',
          caughtErrorsIgnorePattern: '^_',
          varsIgnorePattern: '^_',
        },
      ],
      '@typescript-eslint/consistent-type-imports': [
        'error',
        {
          fixStyle: 'separate-type-imports',
        },
      ],
    },
  },
  eslintPluginPrettierRecommended,
  {
    plugins: { perfectionist },
    rules: {
      'perfectionist/sort-imports': ['error'],
      'perfectionist/sort-exports': ['error'],
    },
  },
  sonarjs.configs.recommended,
  {
    rules: {
      'sonarjs/aws-restricted-ip-admin-access': 'off',
      'sonarjs/no-unused-vars': 'off',
      'sonarjs/todo-tag': 'off',
      'sonarjs/no-skipped-tests': 'off',
      "sonarjs/no-redundant-optional": "off"
    },
  },
  jsxA11y.flatConfigs.recommended,
  {
    rules: {
      'jsx-a11y/label-has-associated-control': 'off',
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
    plugins: { react },
    rules: {
      curly: ['error', 'all'],
      'padding-line-between-statements': [
        'error',
        // Empty line before blocks (if, switch, for)
        { blankLine: 'always', prev: '*', next: 'multiline-block-like' },
        // Empty line after blocks
        { blankLine: 'always', prev: 'multiline-block-like', next: '*' },
      ],
      'react/jsx-curly-brace-presence': [
        'error',
        { 
          props: 'never', 
          children: 'never',
          propElementValues: 'always' 
        }
      ],
      'react/jsx-boolean-value': ['error', 'never'],
    },
  },
  {
    files: ['src/**/*.{ts,tsx}'],
    rules: {
      'no-restricted-imports': ['error', { patterns: [deepFeatureImport, deepModuleImport] }],
    },
  },
  // Flat config replaces a rule instead of merging it, so each block lists every pattern.
  ...[
    ['src/features/**/*.{ts,tsx}', featurePatterns],
    ['src/platform/**/*.{ts,tsx}', platformPatterns],
    ['src/shared/**/*.{ts,tsx}', sharedPatterns],
    ['src/shared/ui-kit/**/*.{ts,tsx}', uiKitPatterns],
  ].map(([glob, patterns]) => ({
    files: [glob],
    rules: { 'no-restricted-imports': ['error', { patterns }] },
  })),
  // A separate rule name, so it adds to the block above instead of replacing it.
  {
    files: ['src/features/*/model/**/*.{ts,tsx}'],
    ignores: ['**/__tests__/**'],
    rules: {
      '@typescript-eslint/no-restricted-imports': ['error', { patterns: [tauriInModel] }],
      // the rule above sees only static imports
      'no-restricted-syntax': [
        'error',
        { selector: 'ImportExpression[source.value=/^@tauri-apps/]', message: tauriInModel.message },
        {
          selector: 'ImportExpression[source.quasis.0.value.raw=/^@tauri-apps/]',
          message: tauriInModel.message,
        },
      ],
    },
  },
];
