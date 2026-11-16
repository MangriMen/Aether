import type { JSX } from 'solid-js';

import { render } from 'solid-js/web';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '@/shared/i18n';

const mocks = vi.hoisted(() => ({
  item: {} as Record<string, unknown>,
  versions: {} as Record<string, unknown>,
  refetchItem: vi.fn(),
  refetchVersions: vi.fn(),
}));

vi.mock('@solidjs/router', () => ({
  useParams: () => ({ contentId: 'c1', versionId: undefined }),
  useMatch: () => () => undefined,
  useNavigate: () => vi.fn(),
}));

vi.mock('@/features/content', async (importOriginal) => ({
  ...(await importOriginal<Record<string, unknown>>()),
  useContent: () => mocks.item,
  useContentVersions: () => mocks.versions,
  useContentContext: () => [{ instanceId: 'i1', providerId: 'p1' }],
}));

vi.mock('../ContentItemPageInfo', () => ({ ContentItemPageInfo: () => null }));

import { ContentItemPage } from '../ContentItemPage';
import { ContentVersionsTab } from '../ContentVersionsTab';

let dispose: (() => void) | undefined;

const mount = (ui: () => JSX.Element) => {
  const container = document.createElement('div');
  document.body.append(container);
  dispose = render(
    () => (
      <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale={LOCALES.En}>
        {ui()}
      </I18nProvider>
    ),
    container,
  );
  return container;
};

const retryButton = () =>
  [...document.querySelectorAll('button')].find((b) =>
    b.textContent.includes('Retry'),
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
  mocks.refetchItem.mockReset();
  mocks.refetchVersions.mockReset();
  mocks.item = {
    data: undefined,
    isError: false,
    isLoading: false,
    isFetching: false,
    refetch: mocks.refetchItem,
  };
  mocks.versions = {
    data: [],
    isError: false,
    isLoading: false,
    isFetching: false,
    refetch: mocks.refetchVersions,
  };
});

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
  vi.unstubAllGlobals();
});

describe('ContentItemPage', () => {
  it('should show a load error with retry when the item failed to load', () => {
    mocks.item = { ...mocks.item, isError: true };

    const container = mount(() => <ContentItemPage />);

    expect(container.querySelector('[role="alert"]')).not.toBeNull();
    expect(container.querySelector('[role="tablist"]')).toBeNull();

    retryButton()?.click();

    expect(mocks.refetchItem).toHaveBeenCalledOnce();
  });

  it('should keep the shown item when only a refresh failed', () => {
    mocks.item = {
      ...mocks.item,
      isError: true,
      data: { id: 'c1', title: 'Item', contentType: 'mod' },
    };

    const container = mount(() => <ContentItemPage />);

    expect(container.querySelector('[role="alert"]')).toBeNull();
  });
});

describe('ContentVersionsTab', () => {
  const item = { id: 'c1', contentType: 'mod' } as never;

  it('should show a load error with retry when versions failed to load', () => {
    mocks.versions = { ...mocks.versions, data: undefined, isError: true };

    const container = mount(() => <ContentVersionsTab item={item} />);

    expect(container.querySelector('[role="alert"]')).not.toBeNull();

    retryButton()?.click();

    expect(mocks.refetchVersions).toHaveBeenCalledOnce();
  });

  it('should not show the skeleton on a background fetch', () => {
    mocks.versions = { ...mocks.versions, isFetching: true };

    const container = mount(() => <ContentVersionsTab item={item} />);

    expect(container.querySelector('[data-animate]')).toBeNull();
  });

  it('should show the skeleton while loading', () => {
    mocks.versions = { ...mocks.versions, data: undefined, isLoading: true };

    const container = mount(() => <ContentVersionsTab item={item} />);

    expect(container.querySelector('[data-animate]')).not.toBeNull();
  });
});
