import { createForm, getInput, isDirty, setInput } from '@formisch/solid';
import { createRoot, createSignal } from 'solid-js';
import { describe, expect, it } from 'vitest';

import type { JavaAndMemorySettingsSchemaInput } from '../../model/javaAndMemorySettingsSchema';

import { JavaAndMemorySettingsSchema } from '../../model/javaAndMemorySettingsSchema';
import { useResetJavaAndMemorySettingsForm } from '../useJavaAndMemorySettingsForm';

const saved: Partial<JavaAndMemorySettingsSchemaInput> = {
  memory: { maximum: 2048 },
  launchArgs: '-Xss1M',
  envVars: 'A=1',
  overrideMemory: true,
  overrideLaunchArgs: true,
  overrideEnvVars: true,
};

// Effects start after the root callback returns, so the scenario runs outside of it.
const mount = () =>
  createRoot((dispose) => {
    const form = createForm({ schema: JavaAndMemorySettingsSchema });
    const [values, setValues] = createSignal(saved);

    const revert = useResetJavaAndMemorySettingsForm(form, values);

    return { form, setValues, revert, dispose };
  });

describe('useResetJavaAndMemorySettingsForm', () => {
  it('should_keep_typed_launch_args_when_the_memory_is_saved', () => {
    const { form, setValues, dispose } = mount();

    setInput(form, { path: ['launchArgs'], input: '-Xss2M -Dtyping' });
    setValues({ ...saved, memory: { maximum: 4096 } });

    expect(getInput(form, { path: ['launchArgs'] })).toBe('-Xss2M -Dtyping');
    expect(getInput(form, { path: ['memory', 'maximum'] })).toBe(4096);

    dispose();
  });

  it('should_reset_the_launch_args_when_the_stored_launch_args_change', () => {
    const { form, setValues, dispose } = mount();

    setInput(form, { path: ['launchArgs'], input: '-Xss2M' });
    setValues({ ...saved, launchArgs: '-Xss4M' });

    expect(getInput(form, { path: ['launchArgs'] })).toBe('-Xss4M');
    expect(isDirty(form, { path: ['launchArgs'] })).toBe(false);

    dispose();
  });

  it('should_put_back_only_the_fields_of_a_failed_save_when_reverting', () => {
    const { form, setValues, revert, dispose } = mount();

    setInput(form, { path: ['launchArgs'], input: '-Xss2M' });
    setInput(form, { path: ['envVars'], input: 'B=typing' });
    // The refresh after the failed save brings the same stored values.
    setValues({ ...saved });

    revert({ launchArgs: '-Xss2M' });

    expect(getInput(form, { path: ['launchArgs'] })).toBe('-Xss1M');
    expect(getInput(form, { path: ['envVars'] })).toBe('B=typing');

    dispose();
  });
});
