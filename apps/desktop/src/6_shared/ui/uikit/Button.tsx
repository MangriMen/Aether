import type { PolymorphicProps } from '@kobalte/core/polymorphic';
import type { VariantProps } from 'class-variance-authority';
import type { Component, JSX, ValidComponent } from 'solid-js';

import * as ButtonPrimitive from '@kobalte/core/button';
import IconMdiLoading from '~icons/mdi/loading';
import { cva } from 'class-variance-authority';
import { splitProps } from 'solid-js';
import { Dynamic } from 'solid-js/web';

import { cn } from '@/shared/lib';

import { DelayedShow } from '../components/DelayedShow';

const buttonVariants = cva(
  'inline-flex items-center justify-center gap-1 rounded-md text-sm font-medium ring-offset-background transition-[color,background-color,border-color,text-decoration-color,fill,stroke,filter,opacity] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-0 disabled:cursor-not-allowed disabled:opacity-50',
  {
    variants: {
      variant: {
        default:
          'bg-primary text-primary-foreground enabled:hover:bg-primary/solid-hover enabled:active:bg-primary/solid-active enabled:active:text-primary-foreground/solid-text-active',

        secondary:
          'border  bg-secondary/secondary text-secondary-foreground enabled:hover:bg-secondary/hover enabled:active:bg-secondary/active enabled:active:text-secondary-foreground/solid-text-active',

        outline:
          'border  bg-transparent enabled:hover:bg-accent/hover enabled:active:bg-accent/active',

        ghost:
          'bg-transparent text-foreground enabled:hover:bg-accent/hover enabled:active:bg-accent/active',

        success:
          'bg-success text-success-foreground transition-all enabled:hover:bg-success/solid-hover enabled:active:bg-success/solid-active enabled:active:text-success-foreground/solid-text-active',

        destructive:
          'bg-destructive text-destructive-foreground transition-all enabled:hover:bg-destructive/solid-hover enabled:active:bg-destructive/solid-active enabled:active:text-destructive-foreground/solid-text-active',

        link: 'text-primary underline-offset-4 enabled:hover:underline',

        ghostWarning:
          'text-warning enabled:hover:bg-warning/control enabled:active:bg-warning/secondary',
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
      <Dynamic component={local.leadingIcon} />
      {local.children}
      <DelayedShow
        when={!local.loading}
        fallback={
          <div
            class={cn(
              'absolute inset-0 flex items-center justify-center m-auto size-full border-none rounded-[inherit] animate-in fade-in-0 ease-in',
              buttonVariants({ variant: local.variant }),
            )}
          >
            <IconMdiLoading class='animate-spin text-xl' />
          </div>
        }
      />
      <Dynamic component={local.trailingIcon} />
    </ButtonPrimitive.Root>
  );
};

export type { ButtonProps };
export { Button, buttonVariants };
