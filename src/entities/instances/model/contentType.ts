export const ContentType = {
  Mod: 'mod',
  DataPack: 'datapack',
  ResourcePack: 'resourcepack',
  ShaderPack: 'shaderpack',
} as const;

export const CONTENT_TYPES = Object.values(ContentType);

export type ContentType = (typeof ContentType)[keyof typeof ContentType];

export const CONTENT_TYPE_TO_TITLE: Record<
  ContentType,
  'mods' | 'dataPacks' | 'resourcePacks' | 'shaders'
> = {
  mod: 'mods',
  datapack: 'dataPacks',
  resourcepack: 'resourcePacks',
  shaderpack: 'shaders',
} as const;
