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
  it('should_show_the_id_and_why_the_instance_cannot_be_read', () => {
    const container = mount(make());

    expect(container.textContent).toContain('old');
    expect(container.textContent).toContain('Unknown game version "../x"');
  });

  it('should_open_its_folder_when_the_button_is_pressed', () => {
    mount(make());

    buttonByText('Open folder')?.click();

    expect(mocks.revealInExplorer).toHaveBeenCalledWith({
      path: '/instances/old',
    });
  });

  it('should_offer_no_folder_when_the_id_names_none', () => {
    mount(make({ dir: null, reason: { code: 'INVALID_ID' } }));

    expect(buttonByText('Open folder')).toBeUndefined();
  });

  it('should_ask_before_deleting_and_say_the_folder_goes_when_it_has_one', () => {
    mount(make());

    buttonByText('Remove')?.click();

    expect(mocks.removeBroken).not.toHaveBeenCalled();
    expect(document.body.textContent).toContain(
      'folder will be deleted from the disk',
    );
  });

  it('should_say_only_the_record_goes_when_the_id_names_no_folder', () => {
    mount(make({ dir: null, reason: { code: 'INVALID_ID' } }));

    buttonByText('Remove')?.click();

    expect(document.body.textContent).toContain('no files are touched');
  });

  it('should_delete_by_the_id_when_the_deletion_is_confirmed', async () => {
    mount(make());

    buttonByText('Remove')?.click();
    const confirm = [...document.querySelectorAll('button')]
      .filter((b) => b.textContent.includes('Remove'))
      .at(-1);
    confirm?.click();
    await Promise.resolve();

    expect(mocks.removeBroken).toHaveBeenCalledWith('old');
  });
});
