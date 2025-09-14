export const ContentType = {
  DataPack: 'datapack',
  Mod: 'mod',
  ResourcePack: 'resourcepack',
  ShaderPack: 'shaderpack',
} as const;

export const CONTENT_TYPES = Object.values(ContentType);

export type ContentType = (typeof ContentType)[keyof typeof ContentType];

export const CONTENT_TYPE_TO_TITLE: Record<
  ContentType,
  'dataPacks' | 'mods' | 'resourcePacks' | 'shaders'
> = {
  datapack: 'dataPacks',
  mod: 'mods',
  resourcepack: 'resourcePacks',
  shaderpack: 'shaders',
} as const;
