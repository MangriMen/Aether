import type { PolymorphicProps } from '@kobalte/core/polymorphic';
import type { VariantProps } from 'class-variance-authority';
import type { Component, JSX, ValidComponent } from 'solid-js';

import * as ButtonPrimitive from '@kobalte/core/button';
import IconMdiLoading from '~icons/mdi/loading';
import { cva } from 'class-variance-authority';
import { splitProps, Show } from 'solid-js';

import { cn } from '@/shared/lib';

const buttonVariants = cva(
  'inline-flex items-center justify-center gap-1 rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-0 disabled:cursor-not-allowed disabled:opacity-50',
  {
    variants: {
      variant: {
        default:
          'bg-primary text-primary-foreground enabled:hover:bg-primary/90',
        destructive:
          'bg-destructive text-destructive-foreground enabled:hover:bg-destructive/90',
        success:
          'bg-success text-success-foreground enabled:hover:bg-success/90',
        outline:
          'border border-input hover:bg-accent enabled:hover:text-accent-foreground',
        secondary:
          'bg-secondary text-secondary-foreground enabled:hover:bg-secondary/80',
        ghost: 'enabled:hover:bg-accent enabled:hover:text-accent-foreground',
        link: 'text-primary underline-offset-4 enabled:hover:underline',
        ghostWarning:
          'text-warning-foreground enabled:hover:bg-warning-foreground enabled:hover:text-secondary',
      },
      size: {
        default: 'h-9 px-4 py-2',
        sm: 'h-8 rounded-md px-3 py-1.5',
        lg: 'h-10 rounded-md px-6 py-3',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  },
);

type ButtonProps<T extends ValidComponent = 'button'> =
  ButtonPrimitive.ButtonRootProps<T> &
    VariantProps<typeof buttonVariants> & {
      loading?: boolean;
      leadingIcon?: Component;
      trailingIcon?: Component;
      class?: string | undefined;
      children?: JSX.Element;
    };

const Button = <T extends ValidComponent = 'button'>(
  props: PolymorphicProps<T, ButtonProps<T>>,
) => {
  const [local, others] = splitProps(props as ButtonProps, [
    'variant',
    'size',
    'loading',
    'leadingIcon',
    'trailingIcon',
    'disabled',
    'class',
    'children',
  ]);

  return (
    <ButtonPrimitive.Root
      class={cn(
        'relative',
        buttonVariants({ variant: local.variant, size: local.size }),
        local.class,
      )}
      disabled={local.disabled || local.loading}
      {...others}
    >
      {local.leadingIcon?.({})}
      {local.children}
      <Show when={local.loading}>
        <div
          class={cn(
            'absolute m-auto size-full',
            buttonVariants({ variant: local.variant }),
          )}
        >
          <IconMdiLoading class='animate-spin text-2xl' />
        </div>
      </Show>
      {local.trailingIcon?.({})}
    </ButtonPrimitive.Root>
  );
};

export type { ButtonProps };
export { Button, buttonVariants };
