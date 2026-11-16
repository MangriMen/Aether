import { createForm, getInput, setInput } from '@formisch/solid';
import { createRoot, createSignal } from 'solid-js';
import { describe, expect, it } from 'vitest';

import type { WindowSettingsSchemaInput } from '../../model/windowSettingsSchema';

import { WindowSettingsSchema } from '../../model/windowSettingsSchema';
import { useResetWindowSettingsFormValues } from '../useWindowSettingsForm';

const saved: Partial<WindowSettingsSchemaInput> = {
  resolution: { width: '960', height: '540' },
  overrideWindowSettings: true,
};

// Effects start after the root callback returns, so the scenario runs outside of it.
const mount = () =>
  createRoot((dispose) => {
    const form = createForm({ schema: WindowSettingsSchema });
    const [values, setValues] = createSignal(saved);

    const revert = useResetWindowSettingsFormValues(form, values);

    return { form, setValues, revert, dispose };
  });

describe('useResetWindowSettingsFormValues', () => {
  it('should_keep_the_typed_height_when_the_width_is_saved', () => {
    const { form, setValues, dispose } = mount();

    setInput(form, { path: ['resolution', 'width'], input: '1280' });
    setInput(form, { path: ['resolution', 'height'], input: '72' });
    setValues({ ...saved, resolution: { width: '1280', height: '540' } });

    expect(getInput(form, { path: ['resolution', 'height'] })).toBe('72');
    expect(getInput(form, { path: ['resolution', 'width'] })).toBe('1280');

    dispose();
  });

  it('should_put_the_stored_resolution_back_when_reverting_a_failed_save', () => {
    const { form, revert, dispose } = mount();

    setInput(form, { path: ['resolution', 'width'], input: '1280' });
    setInput(form, { path: ['resolution', 'height'], input: '720' });

    revert({ resolution: { width: 1280, height: 720 } });

    expect(getInput(form, { path: ['resolution', 'width'] })).toBe('960');
    expect(getInput(form, { path: ['resolution', 'height'] })).toBe('540');

    dispose();
  });
});
