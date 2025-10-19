import type { PolymorphicProps } from '@kobalte/core';
import type { VariantProps } from 'class-variance-authority';
import type { Component, JSX, ValidComponent } from 'solid-js';

import * as ButtonPrimitive from '@kobalte/core/button';
import IconMdiLoading from '~icons/mdi/loading';
import { cva } from 'class-variance-authority';
import { Match, splitProps, Switch } from 'solid-js';

import { cn } from '@/shared/lib';

import { buttonVariants } from './Button';

const SVG_CHILD_FILL_VARIANTS = {
  background: '[&_svg]:fill-background',
  foreground: '[&_svg]:fill-foreground',
  destructive: '[&_svg]:fill-destructive-foreground',
  success: '[&_svg]:fill-success-foreground',
};

const iconButtonVariants = cva('aspect-square', {
  variants: {
    variant: {
      default: SVG_CHILD_FILL_VARIANTS.background,
      destructive: SVG_CHILD_FILL_VARIANTS.destructive,
      success: SVG_CHILD_FILL_VARIANTS.success,
      outline: SVG_CHILD_FILL_VARIANTS.foreground,
      secondary: SVG_CHILD_FILL_VARIANTS.background,
      ghost: SVG_CHILD_FILL_VARIANTS.foreground,
      link: SVG_CHILD_FILL_VARIANTS.foreground,
    },
    size: {
      default: 'size-9 rounded-md text-2xl',
      sm: 'size-8 rounded-md',
      lg: 'size-10 rounded-md',
    },
  },
  defaultVariants: {
    variant: 'default',
    size: 'default',
  },
});

type IconButtonProps<T extends ValidComponent = 'button'> =
  ButtonPrimitive.ButtonRootProps<T> &
    VariantProps<typeof iconButtonVariants> & {
      icon?: Component;
      loading?: boolean;
      class?: string | undefined;
      children?: JSX.Element;
    };

const IconButton = <T extends ValidComponent = 'button'>(
  props: PolymorphicProps<T, IconButtonProps<T>>,
) => {
  const [local, others] = splitProps(props as IconButtonProps, [
    'class',
    'variant',
    'size',
    'icon',
    'loading',
    'disabled',
    'children',
  ]);

  return (
    <ButtonPrimitive.Root
      class={cn(
        buttonVariants({ variant: local.variant, size: local.size }),
        iconButtonVariants({ variant: local.variant, size: local.size }),
        local.class,
      )}
      disabled={local.disabled || local.loading}
      {...others}
    >
      <Switch fallback={local.children}>
        <Match when={local.loading}>
          <IconMdiLoading class='animate-spin' />
        </Match>
        <Match when={local.icon}>{local.icon?.({})}</Match>
      </Switch>
    </ButtonPrimitive.Root>
  );
};

export type { IconButtonProps };
export { IconButton, iconButtonVariants };
