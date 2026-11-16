import { createForm, getInput, setInput } from '@formisch/solid';
import { createRoot, createSignal } from 'solid-js';
import { describe, expect, it } from 'vitest';

import type { HooksSettingsSchemaInput } from '../../model/hooksSettingsSchema';

import { HooksSettingsSchema } from '../../model/hooksSettingsSchema';
import { useResetHooksSettingsFormValues } from '../useHooksSettingsForm';

const saved = { preLaunch: 'a', wrapper: 'b', postExit: 'c' };

// Effects start after the root callback returns, so the scenario runs outside of it.
const mount = () =>
  createRoot((dispose) => {
    const form = createForm({ schema: HooksSettingsSchema });
    const [values, setValues] =
      createSignal<Partial<HooksSettingsSchemaInput>>(saved);

    useResetHooksSettingsFormValues(form, values);

    return { form, setValues, dispose };
  });

describe('useResetHooksSettingsFormValues', () => {
  it('should_keep_an_unsaved_command_when_another_hook_is_saved', () => {
    const { form, setValues, dispose } = mount();

    setInput(form, { path: ['wrapper'], input: '"unclosed' });
    setValues({ ...saved, preLaunch: 'saved' });

    expect(getInput(form, { path: ['wrapper'] })).toBe('"unclosed');
    expect(getInput(form, { path: ['preLaunch'] })).toBe('saved');

    dispose();
  });
});
