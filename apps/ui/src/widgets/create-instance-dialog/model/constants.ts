import type { ModLoader } from '@/entities/minecraft';
import type { Option } from '@/shared/model';

export const LOADERS = [
  { name: 'Vanilla', value: 'vanilla' },
  { name: 'Forge', value: 'forge' },
  { name: 'Fabric', value: 'fabric' },
  { name: 'Quilt', value: 'quilt' },
  { name: 'NeoForge', value: 'neoforge' },
] as const satisfies Option<ModLoader>[];

export const LOADER_VERSION_TYPES = [
  {
    name: 'Stable',
    value: 'stable',
  },
  {
    name: 'Latest',
    value: 'latest',
  },
  {
    name: 'Other',
    value: 'other',
  },
] as const satisfies Option<string>[];
