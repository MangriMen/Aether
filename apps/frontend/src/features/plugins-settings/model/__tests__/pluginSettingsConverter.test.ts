import { describe, expect, it } from 'vitest';

import {
  pluginSettingsToPluginSettingsValues,
  pluginSettingsValuesToEditPluginSettings,
} from '../pluginSettingsConverter';

const ignored = {
  path: { hostPath: '/a', pluginPath: '/cache', readOnly: true },
  reason: 'plugin_reserved',
} as const;

describe('pluginSettingsConverter', () => {
  it('should list the ignored folders with the others when the settings are read', () => {
    const values = pluginSettingsToPluginSettingsValues({
      allowedHosts: [],
      allowedPaths: [{ hostPath: '/b', pluginPath: '/b', readOnly: false }],
      ignoredPaths: [ignored],
    });

    expect(values?.allowedPaths).toEqual([
      { hostPath: '/b', pluginPath: '/b', readOnly: false },
      { ...ignored.path, ignoredReason: 'plugin_reserved' },
    ]);
  });

  it('should send the ignored folders apart from the allowed ones when the form is saved', () => {
    const edit = pluginSettingsValuesToEditPluginSettings({
      allowedPaths: [
        { hostPath: '/b', pluginPath: '/b', readOnly: false },
        { ...ignored.path, ignoredReason: 'plugin_reserved' },
      ],
    });

    expect(edit.allowedPaths).toEqual([
      { hostPath: '/b', pluginPath: '/b', readOnly: false },
    ]);
    expect(edit.ignoredPaths).toEqual([ignored.path]);
  });

  it('should send no ignored folders when the user removed them all', () => {
    const edit = pluginSettingsValuesToEditPluginSettings({
      allowedPaths: [{ hostPath: '/b', pluginPath: '/b' }],
    });

    expect(edit.ignoredPaths).toEqual([]);
  });
});
