import type { PolymorphicProps } from '@kobalte/core/polymorphic';
import type { VariantProps } from 'class-variance-authority';
import type { ValidComponent } from 'solid-js';

import * as ToggleButtonPrimitive from '@kobalte/core/toggle-button';
import { cva } from 'class-variance-authority';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

const toggleVariants = cva(
  'inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-0 disabled:pointer-events-none disabled:opacity-50',
  {
    defaultVariants: {
      size: 'default',
      variant: 'default',
    },
    variants: {
      size: {
        default: 'h-9 px-3',
        lg: 'h-10 px-3',
        sm: 'h-8 px-2',
      },
      variant: {
        default: 'bg-transparent',
        outline: 'border border-input bg-transparent shadow-sm',
      },
    },
  },
);

type ToggleButtonRootProps<T extends ValidComponent = 'button'> = {
  class?: string | undefined;
} & ToggleButtonPrimitive.ToggleButtonRootProps<T> &
  VariantProps<typeof toggleVariants>;

const Toggle = <T extends ValidComponent = 'button'>(
  props: PolymorphicProps<T, ToggleButtonRootProps<T>>,
) => {
  const [local, others] = splitProps(props as ToggleButtonRootProps, [
    'class',
    'variant',
    'size',
  ]);
  return (
    <ToggleButtonPrimitive.Root
      class={cn(
        toggleVariants({ size: local.size, variant: local.variant }),
        local.class,
      )}
      {...others}
    />
  );
};

export type { ToggleButtonRootProps as ToggleProps };
export { Toggle, toggleVariants };
