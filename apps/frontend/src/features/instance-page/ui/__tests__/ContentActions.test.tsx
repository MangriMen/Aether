import { render } from 'solid-js/web';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { ContentFile } from '@/features/content';

import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '@/shared/i18n';

const mocks = vi.hoisted(() => ({
  remove: vi.fn(),
}));

vi.mock('@/features/content', async (importOriginal) => ({
  ...(await importOriginal<Record<string, unknown>>()),
  useRemoveContents: () => ({ mutateAsync: mocks.remove }),
  useEnableContents: () => ({ mutateAsync: vi.fn() }),
  useDisableContents: () => ({ mutateAsync: vi.fn() }),
}));

vi.mock('@/features/instance', async (importOriginal) => ({
  ...(await importOriginal<Record<string, unknown>>()),
  useRevealInExplorer: () => ({ mutateAsync: vi.fn() }),
}));

import { ContentActions } from '../ContentActions';

const content = {
  contentPath: 'mods/a.jar',
  fileName: 'a.jar',
  name: null,
  metadata: null,
  disabled: false,
} as unknown as ContentFile;

let dispose: (() => void) | undefined;

const buttonByText = (text: string) =>
  [...document.querySelectorAll('button')].find((b) =>
    b.textContent.includes(text),
  );

beforeEach(() => {
  vi.stubGlobal(
    'ResizeObserver',
    class {
      observe() {}
      unobserve() {}
      disconnect() {}
    },
  );
  mocks.remove.mockReset().mockResolvedValue(undefined);
  dispose = render(
    () => (
      <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale={LOCALES.En}>
        <ContentActions instanceId='inst' content={content} />
      </I18nProvider>
    ),
    document.body,
  );
});

afterEach(() => {
  vi.unstubAllGlobals();
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

const clickRemoveIcon = () =>
  document
    .querySelector<HTMLButtonElement>('button[class*="destructive"]')!
    .click();

describe('ContentActions remove', () => {
  it('asks before removing and does not call the mutation yet', () => {
    clickRemoveIcon();

    expect(document.querySelector('[role="dialog"]')?.textContent).toContain(
      'a.jar',
    );
    expect(mocks.remove).not.toHaveBeenCalled();
  });

  it('removes the file after the confirmation', () => {
    clickRemoveIcon();
    buttonByText('Remove')!.click();

    expect(mocks.remove).toHaveBeenCalledWith({
      id: 'inst',
      paths: ['mods/a.jar'],
    });
  });

  it('does not remove the file when cancelled', () => {
    clickRemoveIcon();
    buttonByText('Cancel')!.click();

    expect(mocks.remove).not.toHaveBeenCalled();
  });
});
