import { createForm, getInput, setInput } from '@formisch/solid';
import { createRoot, createSignal } from 'solid-js';
import { describe, expect, it } from 'vitest';

import type { GeneralSettingsSchemaInput } from '../../model/generalValidation';

import { GeneralSettingsSchema } from '../../model/generalValidation';
import { useResetGeneralSettingsFormValues } from '../useGeneralSettingsForm';

// Effects start after the root callback returns, so the scenario runs outside of it.
const mount = () =>
  createRoot((dispose) => {
    const form = createForm({ schema: GeneralSettingsSchema });
    const [values, setValues] = createSignal<
      Partial<GeneralSettingsSchemaInput>
    >({ name: 'Saved' });

    useResetGeneralSettingsFormValues(form, values);

    return { form, setValues, dispose };
  });

describe('useResetGeneralSettingsFormValues', () => {
  it('should_keep_the_typed_name_when_the_values_are_rebuilt_with_the_same_name', () => {
    const { form, setValues, dispose } = mount();

    setInput(form, { path: ['name'], input: 'Typing' });
    setValues({ name: 'Saved' });

    expect(getInput(form, { path: ['name'] })).toBe('Typing');

    dispose();
  });

  it('should_reset_the_name_when_the_saved_name_changes', () => {
    const { form, setValues, dispose } = mount();

    setValues({ name: 'Renamed' });

    expect(getInput(form, { path: ['name'] })).toBe('Renamed');

    dispose();
  });
});
