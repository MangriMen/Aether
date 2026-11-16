import { render } from 'solid-js/web';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '@/shared/i18n';

const mocks = vi.hoisted(() => ({
  importers: {} as Record<string, unknown>,
  refetch: vi.fn(),
}));

vi.mock('@/features/plugins', async (importOriginal) => ({
  ...(await importOriginal<Record<string, unknown>>()),
  useImporters: () => mocks.importers,
}));

vi.mock('@/features/instance-import', () => ({
  ImportInstanceView: () => <div data-testid='view' />,
}));

import { ImportInstance } from '../ImportInstance';

let dispose: (() => void) | undefined;

const mount = () => {
  const container = document.createElement('div');
  document.body.append(container);
  dispose = render(
    () => (
      <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale={LOCALES.En}>
        <ImportInstance />
      </I18nProvider>
    ),
    container,
  );
  return container;
};

beforeEach(() => {
  mocks.refetch.mockReset();
  mocks.importers = {
    data: [],
    isError: false,
    isLoading: false,
    refetch: mocks.refetch,
  };
});

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

describe('ImportInstance', () => {
  it('should show a loading state, not the empty message, while loading', () => {
    mocks.importers = { ...mocks.importers, data: undefined, isLoading: true };

    const container = mount();

    expect(
      container.querySelector('[data-testid="importers-loading"]'),
    ).not.toBeNull();
    expect(container.textContent).not.toContain('No import configs');
  });

  it('should show an error with retry when the list failed to load', () => {
    mocks.importers = { ...mocks.importers, data: undefined, isError: true };

    const container = mount();

    expect(container.textContent).not.toContain('No import configs');

    [...container.querySelectorAll('button')]
      .find((b) => b.textContent.includes('Retry'))
      ?.click();

    expect(mocks.refetch).toHaveBeenCalledOnce();
  });

  it('should show the empty message for an empty list', () => {
    expect(mount().textContent).toContain('No import configs');
  });

  it('should show the import view when importers exist', () => {
    mocks.importers = { ...mocks.importers, data: [{ id: 'a' }] };

    expect(mount().querySelector('[data-testid="view"]')).not.toBeNull();
  });
});
