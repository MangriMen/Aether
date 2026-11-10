import { render } from 'solid-js/web';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { BrokenInstance } from '@/features/instance';

import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '@/shared/i18n';

const mocks = vi.hoisted(() => ({
  removeBroken: vi.fn(),
  revealInExplorer: vi.fn(),
}));

vi.mock('@/features/instance', async (importOriginal) => ({
  ...(await importOriginal<Record<string, unknown>>()),
  useRemoveBrokenInstance: () => ({ mutateAsync: mocks.removeBroken }),
  useRevealInExplorer: () => ({ mutateAsync: mocks.revealInExplorer }),
}));

import { BrokenInstanceCard } from '../BrokenInstanceCard';

const make = (extra: Partial<BrokenInstance> = {}): BrokenInstance => ({
  id: 'old',
  dir: '/instances/old',
  origin: 'database',
  reason: { code: 'INVALID_GAME_VERSION', payload: { value: '../x' } },
  ...extra,
});

let dispose: (() => void) | undefined;

const mount = (instance: BrokenInstance) => {
  const container = document.createElement('div');
  document.body.append(container);

  dispose = render(
    () => (
      <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale={LOCALES.En}>
        <BrokenInstanceCard instance={instance} />
      </I18nProvider>
    ),
    container,
  );

  return container;
};

const buttonByText = (text: string) =>
  [...document.querySelectorAll('button')].find((b) =>
    b.textContent.includes(text),
  );

const card = (container: HTMLElement) =>
  container.querySelector<HTMLButtonElement>('button');

const openDetails = (container: HTMLElement) => card(container)?.click();

const openMenu = (container: HTMLElement) =>
  card(container)?.dispatchEvent(
    new MouseEvent('contextmenu', { bubbles: true, cancelable: true }),
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
  mocks.removeBroken.mockReset().mockResolvedValue(null);
  mocks.revealInExplorer.mockReset().mockResolvedValue(undefined);
});

afterEach(() => {
  vi.unstubAllGlobals();
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

describe('BrokenInstanceCard', () => {
  it('should_show_the_id_and_that_it_is_damaged_when_the_card_is_drawn', () => {
    const container = mount(make());

    expect(container.textContent).toContain('old');
    expect(container.textContent).toContain('Damaged');
  });

  it('should_be_one_stop_for_the_arrow_keys_when_the_card_is_drawn', () => {
    const container = mount(make());

    const stops = container.querySelectorAll('button:not([class*="absolute"])');

    expect(stops).toHaveLength(1);
  });

  it('should_show_the_reason_and_the_folder_in_full_when_the_card_is_clicked', () => {
    const container = mount(make());

    openDetails(container);

    expect(document.body.textContent).toContain('Unknown game version "../x"');
    expect(document.body.textContent).toContain('/instances/old');
  });

  it('should_open_its_folder_when_the_button_of_the_details_is_pressed', () => {
    const container = mount(make());
    openDetails(container);

    buttonByText('Open folder')?.click();

    expect(mocks.revealInExplorer).toHaveBeenCalledWith({
      path: '/instances/old',
    });
  });

  it('should_offer_no_folder_when_the_id_names_none', () => {
    const container = mount(
      make({ dir: null, reason: { code: 'INVALID_ID' } }),
    );

    openDetails(container);

    expect(buttonByText('Open folder')).toBeUndefined();
    expect(document.body.textContent).toContain('its id names no folder');
  });

  it('should_ask_before_deleting_and_say_the_folder_goes_when_remove_is_chosen_in_the_details', () => {
    const container = mount(make());
    openDetails(container);

    buttonByText('Remove')?.click();

    expect(mocks.removeBroken).not.toHaveBeenCalled();
    expect(document.body.textContent).toContain(
      'folder will be deleted from the disk',
    );
  });

  it('should_say_only_the_record_goes_when_the_id_names_no_folder', () => {
    const container = mount(
      make({ dir: null, reason: { code: 'INVALID_ID' } }),
    );
    openDetails(container);

    buttonByText('Remove')?.click();

    expect(document.body.textContent).toContain('no files are touched');
  });

  it('should_delete_by_the_id_when_the_deletion_is_confirmed', async () => {
    const container = mount(make());
    openDetails(container);
    buttonByText('Remove')?.click();

    const confirm = [...document.querySelectorAll('button')]
      .filter((b) => b.textContent.includes('Remove'))
      .at(-1);
    confirm?.click();
    await Promise.resolve();

    expect(mocks.removeBroken).toHaveBeenCalledWith('old');
  });

  it('should_offer_the_folder_and_the_removal_when_the_menu_is_opened', async () => {
    const container = mount(make());

    openMenu(container);
    await Promise.resolve();

    const items = [...document.querySelectorAll('[role=menuitem]')].map(
      (item) => item.textContent.trim(),
    );
    expect(items).toEqual(['Open folder', 'Remove']);
  });

  it('should_ask_before_deleting_when_remove_is_chosen_in_the_menu', async () => {
    const container = mount(make());
    openMenu(container);
    await Promise.resolve();

    [...document.querySelectorAll<HTMLElement>('[role=menuitem]')]
      .find((item) => item.textContent.includes('Remove'))
      ?.click();
    await Promise.resolve();

    expect(mocks.removeBroken).not.toHaveBeenCalled();
    expect(document.body.textContent).toContain(
      'folder will be deleted from the disk',
    );
  });
});
