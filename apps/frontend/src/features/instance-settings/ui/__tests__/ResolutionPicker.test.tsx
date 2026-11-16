import { getErrors, setInput } from '@formisch/solid';
import { render } from 'solid-js/web';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '@/shared/i18n';

import type { WindowSettingsSchemaInput } from '../../model/windowSettingsSchema';

import {
  useResetWindowSettingsFormValues,
  useWindowSettingsForm,
} from '../../lib/useWindowSettingsForm';
import { ResolutionPicker } from '../ResolutionPicker';

vi.mock('@tauri-apps/api/window', () => ({
  currentMonitor: () => Promise.resolve(null),
}));

const saved: Partial<WindowSettingsSchemaInput> = {
  resolution: { width: '960', height: '540' },
};

let dispose: (() => void) | undefined;

const mount = () => {
  const container = document.createElement('div');
  document.body.append(container);

  const onSubmit = vi.fn();
  let form!: ReturnType<typeof useWindowSettingsForm>;

  dispose = render(() => {
    form = useWindowSettingsForm();
    useResetWindowSettingsFormValues(form, () => saved);

    return (
      <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale={LOCALES.En}>
        <ResolutionPicker form={form} onSubmit={onSubmit} />
      </I18nProvider>
    );
  }, container);

  const input = (label: string) =>
    container.querySelector<HTMLInputElement>(`input[aria-label="${label}"]`)!;

  return {
    form,
    onSubmit,
    width: input('Window width'),
    height: input('Window height'),
  };
};

const blur = (input: HTMLInputElement) =>
  input.dispatchEvent(new FocusEvent('blur'));

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

describe('ResolutionPicker', () => {
  it.each([' ', 'abc', '0', '70000'])(
    'should_not_submit_when_the_width_is_%j',
    async (value) => {
      const { form, onSubmit, width } = mount();

      setInput(form, { path: ['resolution', 'width'], input: value });
      blur(width);

      await vi.waitFor(() =>
        expect(getErrors(form, { path: ['resolution', 'width'] })).toBeTruthy(),
      );
      expect(onSubmit).not.toHaveBeenCalled();
    },
  );

  it('should_submit_the_numbers_when_both_parts_are_valid', async () => {
    const { form, onSubmit, height } = mount();

    setInput(form, { path: ['resolution', 'width'], input: '1280' });
    setInput(form, { path: ['resolution', 'height'], input: ' 720 ' });
    blur(height);

    await vi.waitFor(() => expect(onSubmit).toHaveBeenCalledWith(1280, 720));
  });

  it('should_not_submit_when_the_width_is_typed_back_to_the_stored_one', async () => {
    const { form, onSubmit, width } = mount();

    setInput(form, { path: ['resolution', 'width'], input: 'abc' });
    blur(width);
    await vi.waitFor(() =>
      expect(getErrors(form, { path: ['resolution', 'width'] })).toBeTruthy(),
    );

    setInput(form, { path: ['resolution', 'width'], input: '960' });
    blur(width);
    await vi.waitFor(() =>
      expect(getErrors(form, { path: ['resolution', 'width'] })).toBeNull(),
    );

    expect(onSubmit).not.toHaveBeenCalled();
  });
});
