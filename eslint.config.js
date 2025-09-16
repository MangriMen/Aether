import globals from 'globals';
import js from '@eslint/js';
import ts from 'typescript-eslint';
import eslintPluginPrettierRecommended from 'eslint-plugin-prettier/recommended';
import tsParser from '@typescript-eslint/parser';
import sonarjs from 'eslint-plugin-sonarjs';
import jsxA11y from 'eslint-plugin-jsx-a11y';
import tailwind from 'eslint-plugin-tailwindcss';
import solid from 'eslint-plugin-solid/configs/recommended';
import perfectionist from 'eslint-plugin-perfectionist';

/** @type {import('eslint').Linter.Config[]} */
export default [
  { files: ['**/*.{js,jsx,ts,tsx}'] },
  {
    ignores: ['*.cjs', 'eslint.config.js', 'dist', '.husky', 'src-tauri'],
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
      curly: 'error',
      'no-debugger': 'warn',
      'no-console': 'warn',
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
    },
  },
  sonarjs.configs.recommended,
  {
    rules: {
      'sonarjs/aws-restricted-ip-admin-access': 'off',
      'sonarjs/no-unused-vars': 'off',
      'sonarjs/todo-tag': 'off',
      'sonarjs/no-skipped-tests': 'off',
    },
  },
  jsxA11y.flatConfigs.recommended,
  {
    rules: {
      'jsx-a11y/label-has-associated-control': 'off',
    },
  },
  ...tailwind.configs['flat/recommended'],
  {
    rules: {
      'tailwindcss/no-custom-classname': [
        'warn',
        {
          cssFiles: ['src/1_app/app.css'],
        },
      ],
    },
  },
  solid,
];
