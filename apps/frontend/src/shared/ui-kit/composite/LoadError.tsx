import type { ComponentProps } from 'solid-js';

import IconMdiAlertCircleOutline from '~icons/mdi/alert-circle-outline';
import { Show, splitProps, type Component } from 'solid-js';

import { cn } from '@/shared/utils';

import { Button } from '../primitives/Button';

export type LoadErrorProps = Omit<ComponentProps<'div'>, 'children'> & {
  message?: string;
  retryLabel?: string;
  onRetry?: () => void;
  /** `block` is centered, for an area that holds nothing else; `inline` is a row among content. */
  variant?: 'block' | 'inline';
};

/** Replacement for content that failed to load, with an optional retry. */
export const LoadError: Component<LoadErrorProps> = (props) => {
  const [local, others] = splitProps(props, [
    'message',
    'retryLabel',
    'onRetry',
    'variant',
    'class',
  ]);

  const retry = () => (
    <Show when={local.onRetry && local.retryLabel}>
      <Button
        variant='secondary'
        size={local.variant === 'inline' ? 'sm' : 'default'}
        class={local.variant === 'inline' ? 'shrink-0' : undefined}
        onClick={() => local.onRetry?.()}
      >
        {local.retryLabel}
      </Button>
    </Show>
  );

  return (
    <Show
      when={local.variant === 'inline'}
      fallback={
        <div
          role='alert'
          class={cn(
            'gap-2 flex flex-col items-center text-center',
            local.class,
          )}
          {...others}
        >
          <p class='text-sm text-destructive'>{local.message}</p>
          {retry()}
        </div>
      }
    >
      <div
        role='alert'
        class={cn(
          `
            gap-3 rounded-md border-destructive/40 bg-destructive/10 px-3 py-2
            flex items-center border
          `,
          local.class,
        )}
        {...others}
      >
        <IconMdiAlertCircleOutline class='size-5 text-destructive shrink-0' />
        <p class='text-sm grow'>{local.message}</p>
        {retry()}
      </div>
    </Show>
  );
};
