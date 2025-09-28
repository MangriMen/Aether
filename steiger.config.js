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
      './src/1_app/**',
      './src/2_pages/**',
      './src/3_widgets/**',
      './src/4_features/**',
      './src/5_entities/**',
      './src/6_shared/**',
    ],
    rules: {
      'fsd/typo-in-layer-name': 'off',
    },
  },
]);
