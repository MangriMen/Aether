export const CONTENT_TYPES = [
  'mod',
  'datapack',
  'resourcepack',
  'shaderpack',
] as const;
export type ContentType = (typeof CONTENT_TYPES)[number];
