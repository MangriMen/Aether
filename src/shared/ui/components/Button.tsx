import MdiLoadingIcon from '@iconify/icons-mdi/loading';
import type { IconifyIcon } from '@iconify-icon/solid';
import { Icon } from '@iconify-icon/solid';
import * as ButtonPrimitive from '@kobalte/core/button';
import type { PolymorphicProps } from '@kobalte/core/polymorphic';
import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { splitProps, Show, createMemo } from 'solid-js';
import type { JSX, ValidComponent } from 'solid-js';

import { cn } from '@/shared/lib';

const buttonVariants = cva(
  'inline-flex items-center justify-center gap-1 rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50',
  {
    variants: {
      variant: {
        default: 'bg-primary text-primary-foreground hover:bg-primary/90',
        destructive:
          'bg-destructive text-destructive-foreground hover:bg-destructive/90',
        success: 'bg-success text-success-foreground hover:bg-success/90',
        outline:
          'border border-input hover:bg-accent hover:text-accent-foreground',
        secondary:
          'bg-secondary text-secondary-foreground hover:bg-secondary/80',
        ghost: 'hover:bg-accent hover:text-accent-foreground',
        link: 'text-primary underline-offset-4 hover:underline',
        ghostWarning:
          'text-warning-foreground hover:bg-warning-foreground hover:text-secondary',
      },
      size: {
        default: 'h-9 px-6 py-2',
        sm: 'h-8 rounded-md px-3',
        lg: 'h-10 rounded-md px-8',
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
      leadingIcon?: IconifyIcon | JSX.Element;
      trailingIcon?: IconifyIcon | JSX.Element;
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

  const isIconifyIcon = (
    value: IconifyIcon | JSX.Element,
  ): value is IconifyIcon =>
    value !== null && typeof value === 'object' && 'body' in value;

  const getIconElement = (icon: IconifyIcon | JSX.Element) => {
    if (!icon || !isIconifyIcon(icon)) {
      return icon;
    }

    return <Icon class='text-xl' icon={icon} />;
  };

  const leadingIcon = createMemo(() => getIconElement(local.leadingIcon));
  const trailingIcon = createMemo(() => getIconElement(local.trailingIcon));

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
      <Show when={local.leadingIcon}>{leadingIcon()}</Show>
      {local.children}
      <Show when={local.loading}>
        <div
          class={cn(
            'absolute m-auto size-full',
            buttonVariants({ variant: local.variant }),
          )}
        >
          <Icon class='animate-spin text-2xl' icon={MdiLoadingIcon} />
        </div>
      </Show>
      <Show when={local.trailingIcon}>{trailingIcon()}</Show>
    </ButtonPrimitive.Root>
  );
};

export type { ButtonProps };
export { Button, buttonVariants };
