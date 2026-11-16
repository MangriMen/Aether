import { render } from 'solid-js/web';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '@/shared/i18n';
import { logError } from '@/shared/log';

import { WindowSettingsForm } from '../WindowSettingsForm';

vi.mock('@tauri-apps/api/window', () => ({
  currentMonitor: () => Promise.resolve(null),
}));
vi.mock('@/shared/log', () => ({ logError: vi.fn() }));

let dispose: (() => void) | undefined;

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
  vi.clearAllMocks();
});

describe('WindowSettingsForm', () => {
  it('should_show_the_stored_width_again_when_the_save_fails', async () => {
    const container = document.createElement('div');
    document.body.append(container);

    const failure = new Error('core refused');
    const onChangePartial = vi.fn(() => Promise.reject(failure));

    dispose = render(
      () => (
        <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale={LOCALES.En}>
          <WindowSettingsForm
            initialValues={() => ({
              resolution: { width: '960', height: '540' },
            })}
            onChangePartial={onChangePartial}
          />
        </I18nProvider>
      ),
      container,
    );

    const width = container.querySelector<HTMLInputElement>(
      'input[aria-label="Window width"]',
    )!;

    width.value = '1280';
    width.dispatchEvent(new InputEvent('input', { bubbles: true }));
    expect(width.value).toBe('1280');
    width.dispatchEvent(new FocusEvent('blur'));

    await vi.waitFor(() => expect(logError).toHaveBeenCalledWith(failure));
    expect(onChangePartial).toHaveBeenCalledWith({
      resolution: { width: 1280, height: 540 },
    });
    expect(width.value).toBe('960');
  });
});
