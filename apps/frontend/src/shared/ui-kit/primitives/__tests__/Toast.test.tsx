import { render } from 'solid-js/web';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { closeToast, showToast, Toaster } from '../Toast';

let dispose: (() => void) | undefined;

const mount = () => {
  dispose = render(() => <Toaster />, document.body);
};

const closeButton = () =>
  document.querySelector<HTMLButtonElement>('li button');

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

describe('showToast', () => {
  it('should_tell_when_the_user_closes_the_toast', async () => {
    mount();
    const onDismiss = vi.fn();

    showToast({ description: 'update ready', persistent: true, onDismiss });
    await expect.poll(closeButton).not.toBeNull();
    closeButton()!.click();

    expect(onDismiss).toHaveBeenCalledTimes(1);
  });

  it('should_not_tell_when_the_toast_is_closed_by_the_code', async () => {
    mount();
    const onDismiss = vi.fn();

    const id = showToast({
      description: 'update ready',
      persistent: true,
      onDismiss,
    });
    await expect.poll(closeButton).not.toBeNull();
    closeToast(id);

    expect(onDismiss).not.toHaveBeenCalled();
  });
});
