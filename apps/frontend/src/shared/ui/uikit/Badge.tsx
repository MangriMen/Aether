import type { VariantProps } from 'class-variance-authority';
import type { Component, ComponentProps } from 'solid-js';

import { cva } from 'class-variance-authority';
import { splitProps } from 'solid-js';

import { cn } from '../../lib';

const badgeVariants = cva(
  `
    rounded-md font-semibold
    focus:ring-ring
    inline-flex items-center border transition-colors
    focus:ring-2 focus:ring-offset-0 focus:outline-none
  `,
  {
    variants: {
      variant: {
        default: 'bg-primary text-primary-foreground border-transparent',
        secondary:
          'bg-secondary/secondary text-secondary-foreground border-transparent',
        outline: 'text-foreground',
        success: 'bg-success text-success-foreground border-transparent',
        warning: 'bg-warning text-warning-foreground border-transparent',
        error: 'bg-error text-error-foreground border-transparent',
      },
      size: {
        sm: 'px-2 py-0 text-xs',
        default: 'px-2.5 py-0.5 text-xs',
        lg: 'px-3 py-1 text-sm',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  },
);

type BadgeProps = ComponentProps<'div'> &
  VariantProps<typeof badgeVariants> & {
    round?: boolean;
  };

const Badge: Component<BadgeProps> = (props) => {
  const [local, others] = splitProps(props, [
    'class',
    'variant',
    'size',
    'round',
  ]);
  return (
    <div
      class={cn(
        badgeVariants({ variant: local.variant, size: local.size }),
        local.round && 'rounded-full',
        local.class,
      )}
      {...others}
    />
  );
};

export type { BadgeProps };
export { Badge, badgeVariants };
