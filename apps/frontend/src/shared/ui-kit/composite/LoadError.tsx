import type { ComponentProps } from 'solid-js';

import { Show, splitProps, type Component } from 'solid-js';

import { cn } from '@/shared/utils';

import { Button } from '../primitives/Button';

export type LoadErrorProps = Omit<ComponentProps<'div'>, 'children'> & {
  message?: string;
  retryLabel?: string;
  onRetry?: () => void;
};

/** Inline replacement for content that failed to load, with an optional retry. */
export const LoadError: Component<LoadErrorProps> = (props) => {
  const [local, others] = splitProps(props, [
    'message',
    'retryLabel',
    'onRetry',
    'class',
  ]);

  return (
    <div
      role='alert'
      class={cn('gap-2 flex flex-col items-center text-center', local.class)}
      {...others}
    >
      <p class='text-sm text-destructive'>{local.message}</p>
      <Show when={local.onRetry && local.retryLabel}>
        <Button variant='secondary' onClick={() => local.onRetry?.()}>
          {local.retryLabel}
        </Button>
      </Show>
    </div>
  );
};
