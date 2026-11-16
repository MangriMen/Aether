import { render } from 'solid-js/web';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '@/shared/i18n';

const mocks = vi.hoisted(() => ({
  maxRam: {} as Record<string, unknown>,
}));

vi.mock('@/features/settings', async (importOriginal) => ({
  ...(await importOriginal<Record<string, unknown>>()),
  useMaxRam: () => mocks.maxRam,
}));

import { OverridableMemoryField } from '../OverridableMemoryField';

let dispose: (() => void) | undefined;

const commit = (value: string, onChange: (v: number) => void) => {
  const container = document.createElement('div');
  document.body.append(container);
  dispose = render(
    () => (
      <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale={LOCALES.En}>
        <OverridableMemoryField value={2048} onChange={onChange} />
      </I18nProvider>
    ),
    container,
  );

  const input = container.querySelector<HTMLInputElement>('input[type="text"]');

  if (!input) {
    throw new Error('no input');
  }

  input.value = value;
  input.dispatchEvent(new FocusEvent('blur'));
};

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

describe('OverridableMemoryField', () => {
  it('should not cap the value at the minimum while RAM is unknown', () => {
    mocks.maxRam = { data: undefined, isError: true, isLoading: false };
    const onChange = vi.fn();

    commit('4096', onChange);

    expect(onChange).toHaveBeenCalledWith(4096);
  });

  it('should cap the value at the total RAM when it is known', () => {
    mocks.maxRam = {
      data: { totalMemory: String(8192 * 1024 * 1024) },
      isError: false,
      isLoading: false,
    };
    const onChange = vi.fn();

    commit('16000', onChange);

    expect(onChange).toHaveBeenCalledWith(8192);
  });
});
