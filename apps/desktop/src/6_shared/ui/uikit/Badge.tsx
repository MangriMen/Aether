import type { VariantProps } from 'class-variance-authority';
import type { Component, ComponentProps } from 'solid-js';

import { cva } from 'class-variance-authority';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

const badgeVariants = cva(
  'inline-flex items-center rounded-md border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-0',
  {
    variants: {
      variant: {
        default:
          'border-transparent bg-new-primary text-new-primary-foreground',
        secondary:
          'border-transparent bg-new-secondary/secondary text-new-secondary-foreground',
        outline: 'text-new-foreground',
        success:
          'border-new-success-foreground bg-new-success text-new-success-foreground',
        warning:
          'border-new-warning-foreground bg-new-warning text-new-warning-foreground',
        error:
          'border-new-error-foreground bg-new-error text-new-error-foreground',
      },
    },
    defaultVariants: {
      variant: 'default',
    },
  },
);

type BadgeProps = ComponentProps<'div'> &
  VariantProps<typeof badgeVariants> & {
    round?: boolean;
  };

const Badge: Component<BadgeProps> = (props) => {
  const [local, others] = splitProps(props, ['class', 'variant', 'round']);
  return (
    <div
      class={cn(
        badgeVariants({ variant: local.variant }),
        local.round && 'rounded-full',
        local.class,
      )}
      {...others}
    />
  );
};

export type { BadgeProps };
export { Badge, badgeVariants };
