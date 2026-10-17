import type { ModLoaderDto } from '@/platform/bindings/minecraft';

export type ModLoader = ModLoaderDto;

// A record, so that a loader added to the bindings is a type error here until it is listed.
const MOD_LOADER_KEYS: Record<ModLoader, true> = {
  vanilla: true,
  forge: true,
  fabric: true,
  quilt: true,
  neoforge: true,
};

export const MOD_LOADERS = Object.keys(MOD_LOADER_KEYS) as readonly ModLoader[];

export const isModLoader = (value: unknown): value is ModLoader =>
  typeof value === 'string' && MOD_LOADERS.includes(value as ModLoader);
