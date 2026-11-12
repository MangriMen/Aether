import * as v from 'valibot';
import { describe, expect, it } from 'vitest';

import { PluginSettingsSchema } from '../pluginSettingsValidation';

describe('PluginSettingsSchema', () => {
  it('should refuse an empty host with a translation key', () => {
    const result = v.safeParse(PluginSettingsSchema, { allowedHosts: ['  '] });

    expect(result.success).toBe(false);
    expect(result.issues?.[0]?.message).toBe('validation.hostRequired');
  });

  it('should trim hosts and let a glob through', () => {
    const result = v.safeParse(PluginSettingsSchema, {
      allowedHosts: [' *.example.com ', '*'],
    });

    expect(result.success).toBe(true);
    expect(result.success && result.output.allowedHosts).toEqual([
      '*.example.com',
      '*',
    ]);
  });
});
