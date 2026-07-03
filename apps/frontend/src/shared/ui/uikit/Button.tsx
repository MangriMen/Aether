import type { PolymorphicProps } from '@kobalte/core/polymorphic';
import type { VariantProps } from 'class-variance-authority';
import type { Component, JSX, ValidComponent } from 'solid-js';

import * as ButtonPrimitive from '@kobalte/core/button';
import { cva } from 'class-variance-authority';
import { splitProps } from 'solid-js';
import { Dynamic } from 'solid-js/web';

import { cn } from '../../lib';

const buttonVariants = cva(
  `
    gap-1 rounded-md text-sm font-medium ring-offset-background
    focus-visible:ring-ring
    inline-flex items-center justify-center
    transition-[color,background-color,border-color,text-decoration-color,fill,stroke,filter,opacity,padding]
    focus-visible:ring-2 focus-visible:ring-offset-0 focus-visible:outline-none
    disabled:cursor-not-allowed disabled:opacity-50
  `,
  {
    variants: {
      variant: {
        default: `
          bg-primary text-primary-foreground
          enabled:hover:bg-primary/solid-hover
          enabled:active:bg-primary/solid-active
          enabled:active:text-primary-foreground/solid-text-active
        `,

        secondary: `
          bg-secondary/secondary text-secondary-foreground
          enabled:hover:bg-secondary/hover
          enabled:active:bg-secondary/active
          enabled:active:text-secondary-foreground/solid-text-active
          border
        `,

        outline: `
          enabled:hover:bg-accent/hover
          enabled:active:bg-accent/active
          border bg-transparent
        `,

        ghost: `
          text-foreground
          enabled:hover:bg-accent/hover
          enabled:active:bg-accent/active
          bg-transparent
        `,

        success: `
          bg-success text-success-foreground
          enabled:hover:bg-success/solid-hover
          enabled:active:bg-success/solid-active
          enabled:active:text-success-foreground/solid-text-active
          transition-all
        `,

        destructive: `
          bg-destructive text-destructive-foreground
          enabled:hover:bg-destructive/solid-hover
          enabled:active:bg-destructive/solid-active
          enabled:active:text-destructive-foreground/solid-text-active
          transition-all
        `,

        link: `
          text-primary underline-offset-4
          enabled:hover:underline
        `,

        warning: `
          bg-warning text-warning-foreground
          enabled:hover:bg-warning/solid-hover
          enabled:active:bg-warning/solid-active
          enabled:active:text-warning-foreground/solid-text-active
        `,

        ghostWarning: `
          text-warning
          enabled:hover:bg-warning/control
          enabled:active:bg-warning/secondary
        `,
      },
      size: {
        default: 'h-9 gap-2 px-4 py-2 leading-4',
        sm: 'h-8 px-3 py-1.5 leading-4',
        lg: 'h-10 px-6 py-2 text-base/5',
        icon: 'size-9',
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
        {
          'animate-pulse-border delay-200': local.loading,
        },
        local.class,
      )}
      style={{
        '--pulse-color': 'var(--foreground)',
      }}
      disabled={local.disabled || local.loading}
      {...others}
    >
      <Dynamic component={local.leadingIcon} />

      {local.children}

      <Dynamic component={local.trailingIcon} />
    </ButtonPrimitive.Root>
  );
};

export type { ButtonProps };
export { Button, buttonVariants };
