export const AtomicContentType = {
  Mod: 'mod',
  DataPack: 'datapack',
  ResourcePack: 'resourcepack',
  ShaderPack: 'shaderpack',
} as const;

export const ATOMIC_CONTENT_TYPES = Object.values(AtomicContentType);

export type AtomicContentType =
  (typeof AtomicContentType)[keyof typeof AtomicContentType];

export const ContentType = {
  Modpack: 'modpack',
  ...AtomicContentType,
} as const;

export const CONTENT_TYPES = Object.values(ContentType);

export type ContentType = (typeof ContentType)[keyof typeof ContentType];

export const CONTENT_TYPE_TO_TITLE: Record<
  ContentType,
  'modpacks' | 'mods' | 'dataPacks' | 'resourcePacks' | 'shaders'
> = {
  modpack: 'modpacks',
  mod: 'mods',
  datapack: 'dataPacks',
  resourcepack: 'resourcePacks',
  shaderpack: 'shaders',
} as const;
