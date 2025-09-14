import type { PolymorphicProps } from '@kobalte/core/polymorphic';
import type { JSX, ValidComponent } from 'solid-js';

import * as CheckboxPrimitive from '@kobalte/core/checkbox';
import { Show, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

type CheckboxRootProps<T extends ValidComponent = 'div'> = {
  class?: string;
  label?: JSX.Element;
} & CheckboxPrimitive.CheckboxRootProps<T>;

const Checkbox = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, CheckboxRootProps<T>>,
) => {
  const [local, others] = splitProps(props as CheckboxRootProps, [
    'label',
    'class',
  ]);

  return (
    <CheckboxPrimitive.Root
      class={cn('items-top group inline-flex items-center gap-2', local.class)}
      {...others}
    >
      <CheckboxPrimitive.Input class='peer' role='checkbox' />
      <CheckboxPrimitive.Control class='size-4 shrink-0 rounded-sm border border-primary ring-offset-background disabled:cursor-not-allowed disabled:opacity-50 peer-focus-visible:outline-none peer-focus-visible:ring-2 peer-focus-visible:ring-ring peer-focus-visible:ring-offset-0 data-[checked]:border-none data-[checked]:bg-primary data-[checked]:text-primary-foreground'>
        <CheckboxPrimitive.Indicator class='data-[checked]:hidden'>
          <svg
            class='size-4'
            fill='none'
            stroke='currentColor'
            stroke-linecap='round'
            stroke-linejoin='round'
            stroke-width='2'
            viewBox='0 0 24 24'
            xmlns='http://www.w3.org/2000/svg'
          >
            <path d='M5 12l11 0' />
          </svg>
        </CheckboxPrimitive.Indicator>
        <CheckboxPrimitive.Indicator class='data-[indeterminate]:hidden'>
          <svg
            class='size-4'
            fill='none'
            stroke='currentColor'
            stroke-linecap='round'
            stroke-linejoin='round'
            stroke-width='2'
            viewBox='0 0 24 24'
            xmlns='http://www.w3.org/2000/svg'
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
