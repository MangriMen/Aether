export const ContentType = {
  Mod: 'mod',
  DataPack: 'datapack',
  ResourcePack: 'resourcepack',
  ShaderPack: 'shaderpack',
} as const;

export const CONTENT_TYPES = Object.values(ContentType);

export type ContentType = (typeof ContentType)[keyof typeof ContentType];
