import type { JSX } from 'solid-js';

import { render } from 'solid-js/web';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { LoadError } from '../LoadError';

let dispose: (() => void) | undefined;

const mount = (ui: () => JSX.Element) => {
  dispose = render(ui, document.body);
};

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.innerHTML = '';
});

describe('LoadError', () => {
  it('shows the message as an alert', () => {
    mount(() => <LoadError message='Failed to load' />);

    expect(document.querySelector('[role="alert"]')?.textContent).toContain(
      'Failed to load',
    );
  });

  it('renders no retry button without a handler', () => {
    mount(() => <LoadError message='Failed' retryLabel='Retry' />);

    expect(document.querySelector('button')).toBeNull();
  });

  it('calls the handler when retry is clicked', () => {
    const onRetry = vi.fn();
    mount(() => (
      <LoadError message='Failed' retryLabel='Retry' onRetry={onRetry} />
    ));

    document.querySelector('button')!.click();

    expect(onRetry).toHaveBeenCalledTimes(1);
  });
});
