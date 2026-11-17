import { createSignal, type Component } from 'solid-js';

import { Button } from '@/shared/ui-kit';

export interface AllowHostButtonProps {
  label: string;
  /** Gives the plugin the host; rejects when the grant failed, which the grant reports itself. */
  onAllow: () => Promise<unknown>;
}

/**
 * Lets a plugin reach the host it was refused, from the error that says so. Once the host is
 * given the button stays off: it is not taken back by pressing again.
 */
export const AllowHostButton: Component<AllowHostButtonProps> = (props) => {
  const [state, setState] = createSignal<'idle' | 'pending' | 'allowed'>(
    'idle',
  );

  const allow = async () => {
    setState('pending');

    try {
      await props.onAllow();
      setState('allowed');
    } catch {
      setState('idle');
    }
  };

  return (
    <Button
      size='sm'
      class='w-fit'
      loading={state() === 'pending'}
      disabled={state() === 'allowed'}
      onClick={() => void allow()}
    >
      {props.label}
    </Button>
  );
};
