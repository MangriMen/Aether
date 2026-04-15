import type { ModLoaderDto } from '@/shared/api';

export type ModLoader = ModLoaderDto;

export const MOD_LOADERS: readonly ModLoader[] = [
  'vanilla',
  'forge',
  'fabric',
  'quilt',
  'neoforge',
] as const;
