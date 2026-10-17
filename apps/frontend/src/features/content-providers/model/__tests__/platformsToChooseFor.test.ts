import { describe, expect, it } from 'vitest';

import type { PlatformProviders } from '../groupProvidersByPlatform';

import { platformsToChooseFor } from '../platformsToChooseFor';

const group = (platform: string, ...ids: string[]): PlatformProviders => ({
  platform,
  providers: ids.map((id) => ({ id, name: id })),
});

describe('platformsToChooseFor', () => {
  it('offers a platform served by several providers', () => {
    const groups = [group('modrinth', 'aether#modrinth', 'p#modrinth')];

    expect(platformsToChooseFor(groups, {})).toEqual(groups);
  });

  it('leaves out a platform with a single provider and no choice', () => {
    expect(
      platformsToChooseFor([group('modrinth', 'aether#modrinth')], {}),
    ).toEqual([]);
  });

  it('keeps a platform with a single provider when a choice is stored', () => {
    const groups = [group('curseforge', 'q#cf')];

    expect(platformsToChooseFor(groups, { curseforge: 'p#cf' })).toEqual(
      groups,
    );
  });

  it('keeps a chosen platform no provider serves any more', () => {
    expect(
      platformsToChooseFor([group('modrinth', 'aether#modrinth')], {
        curseforge: 'p#cf',
      }),
    ).toEqual([group('curseforge')]);
  });

  it('works without settings', () => {
    expect(
      platformsToChooseFor([group('modrinth', 'a', 'b')], undefined),
    ).toHaveLength(1);
  });
});
