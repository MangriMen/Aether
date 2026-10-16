import fsd from '@feature-sliced/steiger-plugin';
import { defineConfig } from 'steiger';

export default defineConfig([
  ...fsd.configs.recommended,
  {
    // disable the `public-api` rule for files in the Shared layer
    files: ['./src/shared/**'],
    rules: {
      'fsd/public-api': 'off',
    },
  },
  {
    files: [
      './src/app/**',
      './src/pages/**',
      './src/widgets/**',
      './src/features/**',
      './src/entities/**',
      './src/shared/**',
    ],
    rules: {
      'fsd/typo-in-layer-name': 'off',
      'fsd/inconsistent-naming': 'off',
    },
  },
  {
    // disabled globally: heuristic rules that do not fit this codebase
    rules: {
      // a slice does not need two consumers to deserve its own folder
      'fsd/insignificant-slice': 'off',
      // app/providers is the conventional name for provider composition
      'fsd/segments-by-purpose': 'off',
      // shared/lib holds flat utilities; grouping them adds no value
      'fsd/shared-lib-grouping': 'off',
    },
  },
]);
