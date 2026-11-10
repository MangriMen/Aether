import { render } from 'solid-js/web';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '@/shared/i18n';

import type { ProgressEvent } from '../../model/progressEvent';

import { cancelProgress } from '../../api/cancelProgress';
import { ProgressCard } from '../ProgressCard';

vi.mock('../../api/cancelProgress', () => ({
  cancelProgress: vi.fn(),
}));

const make = (extra: Partial<ProgressEvent> = {}): ProgressEvent => ({
  event: {
    type: 'pack_download',
    instance_path: '',
    pack_name: 'Pack',
    icon: null,
    pack_id: null,
    pack_version: null,
  },
  progressBarId: 'plan',
  fraction: 0.5,
  message: '',
  stage: 'downloading',
  status: { type: 'running' },
  parentId: null,
  mirrorOf: null,
  bytesDone: null,
  bytesTotal: null,
  speed: null,
  cancel: null,
  ...extra,
});

let dispose: (() => void) | undefined;

const mount = (payload: ProgressEvent) => {
  const container = document.createElement('div');
  document.body.append(container);

  dispose = render(
    () => (
      <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale={LOCALES.En}>
        <ProgressCard payload={payload} />
      </I18nProvider>
    ),
    container,
  );

  return container;
};

const button = (container: HTMLElement) =>
  container.querySelector<HTMLButtonElement>('button');

beforeEach(() => {
  vi.mocked(cancelProgress).mockClear();
});

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

describe('ProgressCard', () => {
  it('calls the work off from a running bar that can be', async () => {
    const container = mount(make({ cancel: 'open' }));

    await expect
      .poll(() => button(container)?.getAttribute('aria-label'))
      .toBe('Cancel');
    expect(button(container)?.disabled).toBe(false);

    button(container)?.click();

    expect(cancelProgress).toHaveBeenCalledWith('plan');
  });

  it('shows the call-off under way until the bar ends', async () => {
    const container = mount(make({ cancel: 'requested' }));

    await expect.poll(() => container.textContent).toContain('Cancelling…');
    expect(button(container)?.disabled).toBe(true);
  });

  it('keeps the button disabled once the work can no longer stop', async () => {
    const container = mount(make({ cancel: 'closed' }));

    await expect
      .poll(() => button(container)?.getAttribute('aria-label'))
      .toBe('Finishing, can no longer be cancelled');
    expect(button(container)?.disabled).toBe(true);
  });

  it('offers no call-off for work that cannot be called off or is over', async () => {
    const plain = mount(make());
    await expect.poll(() => plain.textContent).toContain('Pack');
    expect(button(plain)).toBeNull();
    dispose?.();

    const over = mount(make({ cancel: 'open', status: { type: 'cancelled' } }));
    await expect.poll(() => over.textContent).toContain('Cancelled');
    expect(button(over)).toBeNull();
  });
});
