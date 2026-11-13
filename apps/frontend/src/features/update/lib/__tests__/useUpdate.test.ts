import { createRoot } from 'solid-js';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const install = vi.hoisted(() => ({ mutateAsync: vi.fn() }));

vi.mock('../../api/updateQueries', () => ({
  useInstallUpdate: () => install,
}));

import { useUpdate } from '../useUpdate';

const update = { version: '2.0.0', date: null, body: null };

const updateAndRestart = (status: typeof update | null) =>
  createRoot((dispose) => {
    const { isUpdating, updateAndRestart } = useUpdate();

    return updateAndRestart(status).then((installed) => {
      const updating = isUpdating();
      dispose();
      return { installed, updating };
    });
  });

beforeEach(() => {
  install.mutateAsync.mockReset();
});

describe('useUpdate', () => {
  it('should_say_the_update_was_installed_when_the_install_succeeds', async () => {
    install.mutateAsync.mockResolvedValue(undefined);

    expect(await updateAndRestart(update)).toEqual({
      installed: true,
      updating: false,
    });
  });

  it('should_say_the_update_was_not_installed_when_the_install_fails', async () => {
    install.mutateAsync.mockRejectedValue(new Error('offline'));

    expect(await updateAndRestart(update)).toEqual({
      installed: false,
      updating: false,
    });
  });

  it('should_install_nothing_when_no_update_is_available', async () => {
    expect((await updateAndRestart(null)).installed).toBe(false);
    expect(install.mutateAsync).not.toHaveBeenCalled();
  });
});
