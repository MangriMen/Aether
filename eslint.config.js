import globals from 'globals';
import js from '@eslint/js';
import ts from 'typescript-eslint';
import sonarjs from 'eslint-plugin-sonarjs';
import solid from 'eslint-plugin-solid/configs/recommended';
import tailwind from 'eslint-plugin-tailwindcss';
import importX from 'eslint-plugin-import-x';
import tsParser from '@typescript-eslint/parser';
import jsxA11y from 'eslint-plugin-jsx-a11y';

/** @type {import('eslint').Linter.Config[]} */
export default [
  { files: ['**/*.{js,mjs,jsx,mjsx,ts,tsx,mtsx}'] },
  { ignores: ['*.cjs', 'eslint.config.js', 'dist', 'src-tauri'] },
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
      curly: 'error',
      'no-debugger': 'warn',
    },
  },
  js.configs.recommended,
  ...ts.configs.recommended,
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
      '@typescript-eslint/consistent-type-imports': 'error',
    },
  },
  importX.flatConfigs.recommended,
  importX.flatConfigs.typescript,
  {
    rules: {
      'import-x/no-dynamic-require': 'warn',
      'import-x/no-nodejs-modules': 'warn',
    },
  },
  ...tailwind.configs['flat/recommended'],
  {
    rules: {
      'tailwindcss/no-custom-classname': [
        'warn',
        {
          cssFiles: ['src/**/*.css'],
        },
      ],
    },
  },
  sonarjs.configs.recommended,
  {
    rules: {
      'sonarjs/no-unused-vars': 'off',
      'sonarjs/todo-tag': 'off',
    },
  },
  jsxA11y.flatConfigs.recommended,
  {
    rules: {
      'jsx-a11y/label-has-associated-control': 'off',
    },
  },
  solid,
];
