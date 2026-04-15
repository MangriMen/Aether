import type { ModLoader } from '@/entities/minecraft';
import type { Option } from '@/shared/model';

export const LOADERS: Option<ModLoader>[] = [
  { name: 'Vanilla', value: 'vanilla' },
  { name: 'Forge', value: 'forge' },
  { name: 'Fabric', value: 'fabric' },
  { name: 'Quilt', value: 'quilt' },
  { name: 'NeoForge', value: 'neoforge' },
];

export const LOADER_VERSION_TYPES: Option<string>[] = [
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
] as const;
