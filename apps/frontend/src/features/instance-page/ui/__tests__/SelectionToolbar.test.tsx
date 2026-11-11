import type { RowModel } from '@tanstack/solid-table';

import { render } from 'solid-js/web';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { ContentFile } from '@/features/content';
import type { AppTableFeatures } from '@/shared/ui-kit';

import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '@/shared/i18n';

const mocks = vi.hoisted(() => ({ remove: vi.fn() }));

vi.mock('@/features/content', async (importOriginal) => ({
  ...(await importOriginal<Record<string, unknown>>()),
  useRemoveContents: () => ({ mutateAsync: mocks.remove }),
  useEnableContents: () => ({ mutateAsync: vi.fn() }),
  useDisableContents: () => ({ mutateAsync: vi.fn() }),
}));

import { SelectionToolbar } from '../SelectionToolbar';

const file = (path: string) =>
  ({
    contentPath: path,
    fileName: path,
    name: null,
    metadata: null,
    disabled: false,
  }) as unknown as ContentFile;

const rowsOf = (paths: string[]) =>
  ({
    rows: paths.map((path) => ({ original: file(path) })),
  }) as unknown as RowModel<AppTableFeatures, ContentFile>;

let dispose: (() => void) | undefined;

const mount = (paths: string[], total: number) => {
  const onSelectAll = vi.fn();
  const onClear = vi.fn();
  dispose = render(
    () => (
      <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale={LOCALES.En}>
        <SelectionToolbar
          instanceId='inst'
          selectedRows={rowsOf(paths)}
          totalCount={total}
          onSelectAll={onSelectAll}
          onClear={onClear}
        />
      </I18nProvider>
    ),
    document.body,
  );
  return { onSelectAll, onClear };
};

const buttonByText = (text: string) =>
  [...document.querySelectorAll('button')].find((b) =>
    b.textContent.includes(text),
  );

beforeEach(() => {
  mocks.remove.mockReset().mockResolvedValue(undefined);
});

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

describe('SelectionToolbar', () => {
  it('shows the counts and selects everything on every page', () => {
    const { onSelectAll } = mount(['a', 'b'], 45);

    expect(document.body.textContent).toContain('Selected: 2 of 45');
    buttonByText('Select all (45)')!.click();

    expect(onSelectAll).toHaveBeenCalledTimes(1);
  });

  it('hides select all once everything is selected', () => {
    const { onClear } = mount(['a', 'b'], 2);

    expect(buttonByText('Select all')).toBeUndefined();
    buttonByText('Clear selection')!.click();

    expect(onClear).toHaveBeenCalledTimes(1);
  });

  it('clears the selection after the confirmed removal, not before', async () => {
    const { onClear } = mount(['mods/a.jar', 'mods/b.jar'], 2);

    document
      .querySelector<HTMLButtonElement>('button[class*="destructive"]')!
      .click();

    expect(mocks.remove).not.toHaveBeenCalled();
    expect(onClear).not.toHaveBeenCalled();

    buttonByText('Remove')!.click();
    await Promise.resolve();
    await Promise.resolve();

    expect(mocks.remove).toHaveBeenCalledWith({
      id: 'inst',
      paths: ['mods/a.jar', 'mods/b.jar'],
    });
    expect(onClear).toHaveBeenCalledTimes(1);
  });
});
