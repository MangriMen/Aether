import type { IconifyIcon } from '@iconify-icon/solid';
import type { PolymorphicProps } from '@kobalte/core/polymorphic';
import type { VariantProps } from 'class-variance-authority';
import type { JSX, ValidComponent } from 'solid-js';

import { Icon } from '@iconify-icon/solid';
import MdiLoadingIcon from '@iconify/icons-mdi/loading';
import * as ButtonPrimitive from '@kobalte/core/button';
import { cva } from 'class-variance-authority';
import { createMemo, Show, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

const buttonVariants = cva(
  'inline-flex items-center justify-center gap-1 rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-0 disabled:cursor-not-allowed disabled:opacity-50',
  {
    defaultVariants: {
      size: 'default',
      variant: 'default',
    },
    variants: {
      size: {
        default: 'h-9 px-4 py-2',
        lg: 'h-10 rounded-md px-6 py-3',
        sm: 'h-8 rounded-md px-3 py-1.5',
      },
      variant: {
        default:
          'bg-primary text-primary-foreground enabled:hover:bg-primary/90',
        destructive:
          'bg-destructive text-destructive-foreground enabled:hover:bg-destructive/90',
        ghost: 'enabled:hover:bg-accent enabled:hover:text-accent-foreground',
        ghostWarning:
          'text-warning-foreground enabled:hover:bg-warning-foreground enabled:hover:text-secondary',
        link: 'text-primary underline-offset-4 enabled:hover:underline',
        outline:
          'border border-input hover:bg-accent enabled:hover:text-accent-foreground',
        secondary:
          'bg-secondary text-secondary-foreground enabled:hover:bg-secondary/80',
        success:
          'bg-success text-success-foreground enabled:hover:bg-success/90',
      },
    },
  },
);

type ButtonProps<T extends ValidComponent = 'button'> = {
  children?: JSX.Element;
  class?: string | undefined;
  leadingIcon?: IconifyIcon | JSX.Element;
  loading?: boolean;
  trailingIcon?: IconifyIcon | JSX.Element;
} & ButtonPrimitive.ButtonRootProps<T> &
  VariantProps<typeof buttonVariants>;

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

    return <Icon class='text-lg' icon={icon} />;
  };

  const leadingIcon = createMemo(() => getIconElement(local.leadingIcon));
  const trailingIcon = createMemo(() => getIconElement(local.trailingIcon));

  return (
    <ButtonPrimitive.Root
      class={cn(
        'relative',
        buttonVariants({ size: local.size, variant: local.variant }),
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
