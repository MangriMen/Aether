import type { IconifyIcon } from '@iconify-icon/solid';
import type { PolymorphicProps } from '@kobalte/core';
import type { VariantProps } from 'class-variance-authority';
import type { JSX, ValidComponent } from 'solid-js';

import { Icon } from '@iconify-icon/solid';
import MdiLoadingIcon from '@iconify/icons-mdi/loading';
import * as ButtonPrimitive from '@kobalte/core/button';
import { cva } from 'class-variance-authority';
import { Match, splitProps, Switch } from 'solid-js';

import { cn } from '@/shared/lib';

import { buttonVariants } from './Button';

const SVG_CHILD_FILL_VARIANTS = {
  background: '[&_svg]:fill-background',
  destructive: '[&_svg]:fill-destructive-foreground',
  foreground: '[&_svg]:fill-foreground',
  success: '[&_svg]:fill-success-foreground',
};

const iconButtonVariants = cva('aspect-square', {
  defaultVariants: {
    size: 'default',
    variant: 'default',
  },
  variants: {
    size: {
      default: 'size-9 rounded-md text-2xl',
      lg: 'size-10 rounded-md',
      sm: 'size-8 rounded-md',
    },
    variant: {
      default: SVG_CHILD_FILL_VARIANTS.background,
      destructive: SVG_CHILD_FILL_VARIANTS.destructive,
      ghost: SVG_CHILD_FILL_VARIANTS.foreground,
      link: SVG_CHILD_FILL_VARIANTS.foreground,
      outline: SVG_CHILD_FILL_VARIANTS.foreground,
      secondary: SVG_CHILD_FILL_VARIANTS.background,
      success: SVG_CHILD_FILL_VARIANTS.success,
    },
  },
});

type IconButtonProps<T extends ValidComponent = 'button'> = {
  children?: JSX.Element;
  class?: string | undefined;
  icon?: IconifyIcon;
  loading?: boolean;
} & ButtonPrimitive.ButtonRootProps<T> &
  VariantProps<typeof iconButtonVariants>;

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
        buttonVariants({ size: local.size, variant: local.variant }),
        iconButtonVariants({ size: local.size, variant: local.variant }),
        local.class,
      )}
      disabled={local.disabled || local.loading}
      {...others}
    >
      <Switch fallback={local.children}>
        <Match when={local.loading}>
          <Icon class='animate-spin' icon={MdiLoadingIcon} />
        </Match>
        <Match when={local.icon}>{(icon) => <Icon icon={icon()} />}</Match>
      </Switch>
    </ButtonPrimitive.Root>
  );
};

export type { IconButtonProps };
export { IconButton, iconButtonVariants };
