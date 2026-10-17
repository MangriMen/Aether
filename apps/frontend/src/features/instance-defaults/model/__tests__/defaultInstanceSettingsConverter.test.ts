import { describe, expect, it } from 'vitest';

import type { DefaultInstanceSettings } from '../defaultInstanceSettings';

import { windowSettingsValuesToEditDefaultInstanceSettings } from '../defaultInstanceSettingsConverter';

const settings: DefaultInstanceSettings = {
  launchArgs: [],
  envVars: [],
  memory: { maximum: 2048 },
  window: { forceFullscreen: true, gameResolution: [1280, 720] },
  hooks: { preLaunch: '', wrapper: '', postExit: '' },
};

describe('windowSettingsValuesToEditDefaultInstanceSettings', () => {
  it('keeps fullscreen when only the resolution changes', () => {
    expect(
      windowSettingsValuesToEditDefaultInstanceSettings(settings, {
        resolution: { width: 1920, height: 1080 },
      }),
    ).toEqual({
      window: { forceFullscreen: true, gameResolution: [1920, 1080] },
    });
  });

  it('saves fullscreen when it changes alone', () => {
    expect(
      windowSettingsValuesToEditDefaultInstanceSettings(settings, {
        forceFullscreen: false,
      }),
    ).toEqual({
      window: { forceFullscreen: false, gameResolution: [1280, 720] },
    });
  });

  it('returns an empty edit when nothing changed', () => {
    expect(
      windowSettingsValuesToEditDefaultInstanceSettings(settings, {}),
    ).toEqual({});
  });
});
