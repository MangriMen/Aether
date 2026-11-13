import { createRoot } from 'solid-js';
import { afterEach, describe, expect, it, vi } from 'vitest';

const shown = vi.hoisted(() => vi.fn());

vi.mock('../../api/updateQueries', () => ({
  useCheckUpdate: () => ({
    data: { version: '2.0.0', date: null, body: null },
  }),
}));
vi.mock('../../ui/showUpdateAvailable', () => ({
  showUpdateAvailable: shown,
}));

import {
  setUpdateNotificationStyle,
  UpdateNotificationStyle,
} from '../../model/updateNotificationStyle';
import { useUpdateSync } from '../useUpdateSync';

let dispose: (() => void) | undefined;

afterEach(() => {
  dispose?.();
  dispose = undefined;
  shown.mockClear();
});

describe('useUpdateSync', () => {
  it('should_show_the_waiting_update_when_the_toast_is_switched_on', () => {
    setUpdateNotificationStyle(UpdateNotificationStyle.Banner);
    createRoot((d) => {
      dispose = d;
      useUpdateSync();
    });
    expect(shown).not.toHaveBeenCalled();

    setUpdateNotificationStyle(UpdateNotificationStyle.Toast);

    expect(shown).toHaveBeenCalledTimes(1);
  });
});
