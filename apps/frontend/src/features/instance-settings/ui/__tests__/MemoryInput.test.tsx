import { render } from 'solid-js/web';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '@/shared/i18n';

import { MemoryInput } from '../MemoryInput';

let dispose: (() => void) | undefined;

beforeEach(() => {
  vi.stubGlobal(
    'ResizeObserver',
    class {
      observe() {}
      unobserve() {}
      disconnect() {}
    },
  );
});

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
  vi.unstubAllGlobals();
});

describe('MemoryInput', () => {
  it('should show the controlled value when the owner clamps the typed one to it', () => {
    const container = document.createElement('div');
    document.body.append(container);
    // The owner ignores the typed value, as a clamp to the current maximum does.
    const onChange = vi.fn();

    dispose = render(
      () => (
        <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale={LOCALES.En}>
          <MemoryInput
            value={[2048]}
            minValue={512}
            maxValue={2048}
            onChange={onChange}
          />
        </I18nProvider>
      ),
      container,
    );

    const input =
      container.querySelector<HTMLInputElement>('input[type="text"]')!;

    input.value = '999999';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    input.dispatchEvent(new FocusEvent('blur'));

    expect(onChange).toHaveBeenCalledWith([999999]);
    expect(input.value).toBe('2048');
  });
});
