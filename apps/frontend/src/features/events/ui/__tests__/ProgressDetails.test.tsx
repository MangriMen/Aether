import { render } from 'solid-js/web';
import { afterEach, describe, expect, it } from 'vitest';

import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '@/shared/i18n';

import type { ProgressEvent } from '../../model/progressEvent';

import { buildProgressIndex } from '../../model/progressTree';
import { ProgressDetails } from '../ProgressDetails';

const make = (
  id: string,
  parentId: string | null,
  extra: Partial<ProgressEvent> = {},
): ProgressEvent => ({
  event: { type: 'file_download', file_name: id },
  progressBarId: id,
  fraction: 0.5,
  message: '',
  stage: 'downloading',
  status: { type: 'running' },
  parentId,
  mirrorOf: null,
  bytesDone: null,
  bytesTotal: null,
  speed: null,
  ...extra,
});

const done = (id: string, parentId: string | null): ProgressEvent =>
  make(id, parentId, { fraction: 1, status: { type: 'done' } });

let dispose: (() => void) | undefined;

const mount = (bars: ProgressEvent[]) => {
  const container = document.createElement('div');
  document.body.append(container);

  dispose = render(
    () => (
      <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale={LOCALES.En}>
        <ProgressDetails index={() => buildProgressIndex(bars)} />
      </I18nProvider>
    ),
    container,
  );

  return container;
};

const openAll = (container: HTMLElement) => {
  for (let round = 0; round < 3; round += 1) {
    container
      .querySelectorAll<HTMLElement>(
        'button[aria-label="Show details"][aria-expanded="false"]',
      )
      .forEach((button) => button.click());
  }
};

const titles = (container: HTMLElement, names: string[]) =>
  [...container.querySelectorAll('span.truncate')]
    .map((el) => el.textContent ?? '')
    .filter((text) => names.includes(text));

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

describe('ProgressDetails', () => {
  it('says there is nothing to show when there are no bars', async () => {
    const container = mount([]);

    await expect
      .poll(() => container.textContent)
      .toContain('No downloads right now');
  });

  it('keeps the children where they were while the parent works', async () => {
    const container = mount([
      make('root', null),
      make('a', 'root'),
      done('b', 'root'),
      make('c', 'root'),
    ]);

    await expect.poll(() => container.textContent).toContain('root');
    openAll(container);

    await expect
      .poll(() => titles(container, ['a', 'b', 'c']))
      .toEqual(['a', 'b', 'c']);
    expect(container.textContent).not.toContain('Completed: 1');
  });

  it('folds the finished children once the parent is over', async () => {
    const container = mount([
      done('root', null),
      make('a', 'root', { status: { type: 'failed', message: 'boom' } }),
      done('b', 'root'),
    ]);

    await expect.poll(() => container.textContent).toContain('root');
    openAll(container);

    await expect.poll(() => container.textContent).toContain('Completed: 1');
    expect(titles(container, ['a', 'b'])).toEqual(['a']);
  });
});
