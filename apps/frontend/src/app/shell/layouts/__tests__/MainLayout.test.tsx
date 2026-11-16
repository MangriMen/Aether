import { createMemoryHistory, MemoryRouter, Route } from '@solidjs/router';
import { createSignal } from 'solid-js';
import { render } from 'solid-js/web';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '@/shared/i18n';

vi.mock('@/features/account', () => ({
  CreateOfflineAccountDialog: () => null,
}));
vi.mock('@/features/instance-create', () => ({
  CreateInstanceDialog: () => null,
}));
vi.mock('../../sidebar/AppSidebar', () => ({
  AppSidebar: () => <nav data-testid='sidebar' />,
}));

import { MainLayout } from '../MainLayout';

let dispose: (() => void) | undefined;

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
  vi.restoreAllMocks();
});

describe('MainLayout', () => {
  it('should keep the sidebar when a page throws and recover on reset', async () => {
    const [broken, setBroken] = createSignal(true);

    const Page = () => {
      if (broken()) {
        throw new Error('boom');
      }

      return <div data-testid='page'>ok</div>;
    };

    const container = document.createElement('div');
    document.body.append(container);
    vi.spyOn(console, 'error').mockImplementation(() => {});

    dispose = render(
      () => (
        <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale={LOCALES.En}>
          <MemoryRouter
            root={(props) => <MainLayout>{props.children}</MainLayout>}
          >
            <Route path='/' component={Page} />
          </MemoryRouter>
        </I18nProvider>
      ),
      container,
    );
    await Promise.resolve();

    expect(container.querySelector('[data-testid="sidebar"]')).not.toBeNull();
    expect(container.querySelector('[data-testid="page"]')).toBeNull();
    expect(container.querySelector('button')).not.toBeNull();

    setBroken(false);
    container.querySelector('button')?.click();
    await Promise.resolve();

    expect(container.querySelector('[data-testid="page"]')).not.toBeNull();
    expect(container.querySelector('[data-testid="sidebar"]')).not.toBeNull();
  });

  it('should go home on reset when a page other than home throws', async () => {
    const history = createMemoryHistory();
    history.set({ value: '/broken', scroll: false, replace: true });

    const container = document.createElement('div');
    document.body.append(container);
    vi.spyOn(console, 'error').mockImplementation(() => {});

    dispose = render(
      () => (
        <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale={LOCALES.En}>
          <MemoryRouter
            history={history}
            root={(props) => <MainLayout>{props.children}</MainLayout>}
          >
            <Route
              path='/'
              component={() => <div data-testid='home'>ok</div>}
            />
            <Route
              path='/broken'
              component={() => {
                throw new Error('boom');
              }}
            />
          </MemoryRouter>
        </I18nProvider>
      ),
      container,
    );
    await Promise.resolve();

    expect(container.querySelector('[data-testid="home"]')).toBeNull();
    expect(container.querySelector('button')).not.toBeNull();

    container.querySelector('button')?.click();
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(container.querySelector('[data-testid="home"]')).not.toBeNull();
    expect(container.querySelector('[data-testid="sidebar"]')).not.toBeNull();
  });
});
