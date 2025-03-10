import type { ContentType } from '@/entities/instances';

export const CONTENT_TYPE_TO_TITLE: Record<
  ContentType,
  'mods' | 'dataPacks' | 'resourcePacks' | 'shaders'
> = {
  mod: 'mods',
  datapack: 'dataPacks',
  resourcepack: 'resourcePacks',
  shaderpack: 'shaders',
} as const;
