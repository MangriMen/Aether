import type { PolymorphicProps } from '@kobalte/core/polymorphic';
import type { VariantProps } from 'class-variance-authority';
import type { Component, JSX, ValidComponent } from 'solid-js';

import * as ButtonPrimitive from '@kobalte/core/button';
import IconMdiLoading from '~icons/mdi/loading';
import { cva } from 'class-variance-authority';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

import { DelayedShow } from '../components/DelayedShow';

const buttonVariants = cva(
  'inline-flex items-center justify-center gap-1 rounded-md text-sm font-medium ring-offset-new-background transition-[color,background-color,border-color,text-decoration-color,fill,stroke,filter] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-new-ring focus-visible:ring-offset-0 disabled:cursor-not-allowed disabled:opacity-50',
  {
    variants: {
      variant: {
        default:
          'bg-new-primary text-new-primary-foreground enabled:hover:bg-new-primary/solid-hover enabled:active:bg-new-primary/solid-active enabled:active:text-new-primary-foreground/solid-text-active',

        secondary:
          'border border-new-border bg-new-secondary/secondary text-new-secondary-foreground enabled:hover:bg-new-secondary/hover enabled:active:bg-new-secondary/active enabled:active:text-new-secondary-foreground/solid-text-active',

        outline:
          'border border-new-border bg-transparent enabled:hover:bg-new-accent/control enabled:active:bg-new-accent/secondary',

        ghost:
          'bg-transparent text-new-foreground enabled:hover:bg-new-accent/control enabled:active:bg-new-accent/secondary',

        success:
          'bg-new-success text-new-success-foreground transition-all enabled:hover:bg-new-success/solid-hover enabled:active:bg-new-success/solid-active enabled:active:text-new-success-foreground/solid-text-active',

        destructive:
          'bg-new-destructive text-new-destructive-foreground transition-all enabled:hover:bg-new-destructive/solid-hover enabled:active:bg-new-destructive/solid-active enabled:active:text-new-destructive-foreground/solid-text-active',

        link: 'text-new-primary underline-offset-4 enabled:hover:underline',

        ghostWarning:
          'text-new-warning enabled:hover:bg-new-warning/control enabled:active:bg-new-warning/secondary',
      },
      size: {
        default: 'h-9 px-4 py-2 text-sm leading-4',
        sm: 'h-8 rounded-md px-3 py-1.5 text-sm leading-4',
        lg: 'h-10 rounded-md px-6 py-2 text-base leading-5',
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
        local.class,
      )}
      disabled={local.disabled || local.loading}
      {...others}
    >
      {local.leadingIcon?.({})}
      {local.children}
      <DelayedShow when={local.loading}>
        <div
          class={cn(
            'absolute m-auto size-full',
            buttonVariants({ variant: local.variant }),
          )}
        >
          <IconMdiLoading class='animate-spin text-xl' />
        </div>
      </DelayedShow>
      {local.trailingIcon?.({})}
    </ButtonPrimitive.Root>
  );
};

export type { ButtonProps };
export { Button, buttonVariants };
