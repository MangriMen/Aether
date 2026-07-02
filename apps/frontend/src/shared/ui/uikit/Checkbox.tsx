import type { PolymorphicProps } from '@kobalte/core/polymorphic';
import type { JSX, ValidComponent } from 'solid-js';

import * as CheckboxPrimitive from '@kobalte/core/checkbox';
import { Show, splitProps } from 'solid-js';

import { cn } from '../../lib';

type CheckboxRootProps<T extends ValidComponent = 'div'> =
  CheckboxPrimitive.CheckboxRootProps<T> & {
    label?: JSX.Element;
    class?: string;
  };

const Checkbox = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, CheckboxRootProps<T>>,
) => {
  const [local, others] = splitProps(props as CheckboxRootProps, [
    'label',
    'class',
  ]);

  return (
    <CheckboxPrimitive.Root
      class={cn('group gap-2 inline-flex items-center', local.class)}
      {...others}
    >
      <CheckboxPrimitive.Input role='checkbox' class='peer' />
      <CheckboxPrimitive.Control
        class='
          size-4 rounded-sm border-border-dynamic/15 bg-secondary/control
          ring-offset-background
          peer-focus-visible:ring-ring
          hover:border-border-dynamic/25 hover:bg-secondary/hover
          data-checked:bg-primary data-checked:text-primary-foreground
          data-checked:hover:bg-primary/solid-hover
          shrink-0 border transition-colors
          peer-focus-visible:ring-2 peer-focus-visible:ring-offset-0
          peer-focus-visible:outline-none
          active:scale-95
          disabled:cursor-not-allowed disabled:opacity-50
          data-checked:border-none data-checked:brightness-110
        '
      >
        <CheckboxPrimitive.Indicator class='data-checked:hidden'>
          <svg
            xmlns='http://www.w3.org/2000/svg'
            viewBox='0 0 24 24'
            fill='none'
            stroke='currentColor'
            stroke-width='2'
            stroke-linecap='round'
            stroke-linejoin='round'
            class='size-4'
          >
            <path d='M5 12l11 0' />
          </svg>
        </CheckboxPrimitive.Indicator>
        <CheckboxPrimitive.Indicator class='data-indeterminate:hidden'>
          <svg
            xmlns='http://www.w3.org/2000/svg'
            viewBox='0 0 24 24'
            fill='none'
            stroke='currentColor'
            stroke-width='2'
            stroke-linecap='round'
            stroke-linejoin='round'
            class='size-4'
          >
            <path d='M5 12l5 5l10 -10' />
          </svg>
        </CheckboxPrimitive.Indicator>
      </CheckboxPrimitive.Control>
      <Show when={local.label}>
        <CheckboxPrimitive.Label>{local.label}</CheckboxPrimitive.Label>
      </Show>
    </CheckboxPrimitive.Root>
  );
};

export { Checkbox };
export type { CheckboxRootProps };
