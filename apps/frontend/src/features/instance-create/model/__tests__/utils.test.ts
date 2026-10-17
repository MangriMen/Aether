import { describe, expect, it } from 'vitest';

import type { MappedLoaderManifest } from '@/features/minecraft';

import { getLoaderVersionsForGameVersion } from '../utils';

const loader = (id: string) => ({
  id,
  url: `https://meta/${id}`,
  stable: true,
});

const quilt: MappedLoaderManifest = {
  gameVersions: {
    '1.21': { id: '1.21', stable: true, loaders: [], versionGroup: 'v1' },
    '26.3': { id: '26.3', stable: true, loaders: [], versionGroup: 'v2' },
    '1.20': {
      id: '1.20',
      stable: true,
      loaders: [loader('own')],
      versionGroup: null,
    },
    '1.19': { id: '1.19', stable: true, loaders: [], versionGroup: 'gone' },
  },
  versionGroups: {
    v1: { id: 'v1', loaders: [loader('0.29.0')] },
    v2: { id: 'v2', loaders: [loader('0.31.0')] },
  },
};

const quiltLoadersOf = (gameVersion: string) =>
  getLoaderVersionsForGameVersion('quilt', gameVersion, { quilt }).map(
    (item) => item.id,
  );

describe('getLoaderVersionsForGameVersion', () => {
  it('takes the loaders of the version group when a game version has one', () => {
    expect(quiltLoadersOf('1.21')).toEqual(['0.29.0']);
    expect(quiltLoadersOf('26.3')).toEqual(['0.31.0']);
  });

  it('takes the own loaders when a game version has no group', () => {
    expect(quiltLoadersOf('1.20')).toEqual(['own']);
  });

  it('returns nothing when the group or the game version is unknown', () => {
    expect(quiltLoadersOf('1.19')).toEqual([]);
    expect(quiltLoadersOf('1.0')).toEqual([]);
  });
});
