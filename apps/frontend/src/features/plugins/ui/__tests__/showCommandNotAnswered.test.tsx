import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { ShowToastParams } from '@/shared/ui-kit';

const toast = vi.hoisted(() => ({
  show: vi.fn<(params: ShowToastParams) => number>(),
  close: vi.fn(),
}));

vi.mock('@/shared/ui-kit', () => ({
  showToast: toast.show,
  closeToast: toast.close,
}));

const text = { title: 'packwiz got no answer', description: 'nothing was run' };

/** A fresh module each time: which toasts are up is module state. */
const load = async () => {
  vi.resetModules();
  return import('../showCommandNotAnswered');
};

describe('showCommandNotAnswered', () => {
  beforeEach(() => {
    toast.show.mockReset();
    toast.close.mockReset();
    let next = 0;
    toast.show.mockImplementation(() => ++next);
  });

  it('should_show_one_persistent_warning_with_the_given_text', async () => {
    const { showCommandNotAnswered } = await load();

    showCommandNotAnswered('packwiz', text);

    expect(toast.show).toHaveBeenCalledTimes(1);
    const params = toast.show.mock.lastCall![0];
    expect(params.persistent).toBe(true);
    expect(params.variant).toBe('warning');
    expect(params.title).toBe(text.title);
    expect(params.description).toBe(text.description);
  });

  it('should_not_stack_a_second_message_for_the_same_plugin', async () => {
    const { showCommandNotAnswered } = await load();

    showCommandNotAnswered('packwiz', text);
    showCommandNotAnswered('packwiz', text);

    expect(toast.show).toHaveBeenCalledTimes(1);
  });

  it('should_show_a_message_for_each_plugin', async () => {
    const { showCommandNotAnswered } = await load();

    showCommandNotAnswered('a', text);
    showCommandNotAnswered('b', text);

    expect(toast.show).toHaveBeenCalledTimes(2);
  });

  it('should_allow_a_new_message_once_the_user_closed_the_old_one', async () => {
    const { showCommandNotAnswered } = await load();
    showCommandNotAnswered('packwiz', text);

    toast.show.mock.lastCall![0].onDismiss!();
    showCommandNotAnswered('packwiz', text);

    expect(toast.show).toHaveBeenCalledTimes(2);
  });
});

describe('closeCommandNotAnswered', () => {
  beforeEach(() => {
    toast.show.mockReset();
    toast.close.mockReset();
    toast.show.mockReturnValue(7);
  });

  it('should_close_the_message_when_the_plugin_asks_again', async () => {
    const { closeCommandNotAnswered, showCommandNotAnswered } = await load();
    showCommandNotAnswered('packwiz', text);

    closeCommandNotAnswered('packwiz');

    expect(toast.close).toHaveBeenCalledWith(7);
  });

  it('should_do_nothing_when_there_is_no_message', async () => {
    const { closeCommandNotAnswered } = await load();

    closeCommandNotAnswered('packwiz');

    expect(toast.close).not.toHaveBeenCalled();
  });
});
