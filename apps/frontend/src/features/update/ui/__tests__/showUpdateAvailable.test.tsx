import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { ShowToastParams } from '@/shared/ui-kit';

const toast = vi.hoisted(() => ({
  show: vi.fn<(params: ShowToastParams) => number>(() => 1),
  close: vi.fn(),
}));
const dialog = vi.hoisted(() => ({
  onOk: undefined as (() => unknown) | undefined,
}));
const update = vi.hoisted(() => ({ updateAndRestart: vi.fn() }));
const buttons = vi.hoisted(() => ({
  props: [] as { disabled?: boolean; onClick?: () => void }[],
}));
const games = vi.hoisted(() => ({
  names: [] as string[],
  carryOn: false,
}));
const settings = vi.hoisted(() => ({ open: vi.fn() }));

vi.mock('@/shared/ui-kit', () => ({
  Button: (props: { disabled?: boolean; onClick?: () => void }) => {
    buttons.props.push(props);
    return null;
  },
  showToast: toast.show,
  closeToast: toast.close,
}));
vi.mock('@/shared/i18n', () => ({
  useTranslation: () => [{ t: (key: string) => key }],
}));
vi.mock('@/features/settings', () => ({
  SettingsTab: { Update: 'update' },
  useSettingsSearchParams: () => ({ open: settings.open }),
}));
vi.mock('../../lib/useRunningGames', () => ({
  useRunningGames: () => ({
    names: () => games.names,
    isRunning: () => games.names.length > 0,
    keepsGames: () => games.names.length > 0 && games.carryOn,
    isBlocked: () => games.names.length > 0 && !games.carryOn,
  }),
}));
vi.mock('../RunningGamesHint', () => ({ RunningGamesHint: () => null }));
vi.mock('../../lib/useUpdate', () => ({
  useUpdate: () => ({
    isUpdating: () => false,
    updateAndRestart: update.updateAndRestart,
  }),
}));
vi.mock('../UpdateAndRestartWarningDialog', () => ({
  UpdateAndRestartWarningDialog: (props: { onOk: () => unknown }) => {
    // A stand-in for the dialog: the handler is taken once, as the toast hands it over.
    // eslint-disable-next-line solid/reactivity
    dialog.onOk = props.onOk;
    return null;
  },
}));

const status = { version: '2.0.0', date: null, body: null };

/** A fresh module each time: whether a toast is up is module state. */
const load = async () => {
  vi.resetModules();
  return (await import('../showUpdateAvailable')).showUpdateAvailable;
};

const lastToast = () => toast.show.mock.lastCall![0];

beforeEach(() => {
  toast.show.mockClear();
  toast.close.mockClear();
  update.updateAndRestart.mockReset();
  dialog.onOk = undefined;
  games.names = [];
  games.carryOn = false;
  buttons.props = [];
});

describe('showUpdateAvailable', () => {
  it('should_keep_the_toast_when_the_install_fails', async () => {
    const show = await load();
    update.updateAndRestart.mockResolvedValue(false);

    show(status);
    await dialog.onOk!();
    show(status);

    expect(toast.close).not.toHaveBeenCalled();
    expect(toast.show).toHaveBeenCalledTimes(1);
  });

  it('should_close_the_toast_and_allow_the_next_one_when_the_install_succeeds', async () => {
    const show = await load();
    update.updateAndRestart.mockResolvedValue(true);

    show(status);
    await dialog.onOk!();
    show(status);

    expect(toast.close).toHaveBeenCalledWith(1);
    expect(toast.show).toHaveBeenCalledTimes(2);
  });

  it('should_allow_the_next_toast_when_the_user_closes_this_one', async () => {
    const show = await load();

    show(status);
    lastToast().onDismiss!();
    show(status);

    expect(toast.show).toHaveBeenCalledTimes(2);
  });

  it('should_disable_the_update_button_when_a_game_runs', async () => {
    const show = await load();
    games.names = ['Alpha'];

    show(status);

    expect(buttons.props[0]?.disabled).toBe(true);
  });

  it('should_enable_the_update_button_when_the_games_carry_on_across_the_restart', async () => {
    const show = await load();
    games.names = ['Alpha'];
    games.carryOn = true;
    update.updateAndRestart.mockResolvedValue(true);

    show(status);
    await dialog.onOk!();

    expect(buttons.props[0]?.disabled).toBe(false);
    expect(update.updateAndRestart).toHaveBeenCalledWith(status, true);
  });

  it('should_enable_the_update_button_when_no_game_runs', async () => {
    const show = await load();

    show(status);

    expect(buttons.props[0]?.disabled).toBe(false);
  });
});
