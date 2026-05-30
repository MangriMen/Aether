export const ContentType = {
  Modpack: 'modpack',
  Mod: 'mod',
  DataPack: 'datapack',
  ResourcePack: 'resourcepack',
  ShaderPack: 'shaderpack',
} as const;

export type ContentType = (typeof ContentType)[keyof typeof ContentType];

export type AtomicContentType = Exclude<
  ContentType,
  typeof ContentType.Modpack
>;

export const CONTENT_TYPES = Object.values(
  ContentType,
) as readonly ContentType[];

export const ATOMIC_CONTENT_TYPES: readonly AtomicContentType[] = [
  ContentType.Mod,
  ContentType.DataPack,
  ContentType.ResourcePack,
  ContentType.ShaderPack,
];

export const CONTENT_TYPE_TO_TITLE = {
  [ContentType.Mod]: 'mods',
  [ContentType.DataPack]: 'dataPacks',
  [ContentType.ResourcePack]: 'resourcePacks',
  [ContentType.ShaderPack]: 'shaders',
  [ContentType.Modpack]: 'modpacks',
} as const satisfies Record<ContentType, string>;

export const isContentType = (value: unknown): value is ContentType =>
  typeof value === 'string' && CONTENT_TYPES.includes(value as ContentType);

export const isAtomicContentType = (
  value: unknown,
): value is AtomicContentType =>
  isContentType(value) && value !== ContentType.Modpack;
