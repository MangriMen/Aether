import { Icon, IconifyIcon } from '@iconify-icon/solid';
import { PolymorphicProps } from '@kobalte/core';
import * as ButtonPrimitive from '@kobalte/core/button';
import { cva } from 'class-variance-authority';
import type { VariantProps } from 'class-variance-authority';
import { Show, splitProps, ValidComponent } from 'solid-js';

import { cn } from '@/shared/lib';

import { ButtonProps, buttonVariants } from './Button';

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
      default: 'size-10 rounded-md',
      sm: 'size-9 rounded-md',
      lg: 'size-11 rounded-md',
    },
  },
  defaultVariants: {
    variant: 'default',
    size: 'default',
  },
});

type OnlyIconButtonProps = VariantProps<typeof iconButtonVariants>;

type IconButtonProps<T extends ValidComponent = 'button'> =
  OnlyIconButtonProps &
    ButtonProps<T> & {
      icon?: IconifyIcon;
    };

const IconButton = <T extends ValidComponent = 'button'>(
  props: PolymorphicProps<T, IconButtonProps<T>>,
) => {
  const [local, others] = splitProps(props as IconButtonProps, [
    'class',
    'variant',
    'size',
    'icon',
    'children',
  ]);

  return (
    <ButtonPrimitive.Root
      class={cn(
        buttonVariants({ variant: local.variant, size: local.size }),
        iconButtonVariants({ variant: local.variant, size: local.size }),
        local.class,
      )}
      {...others}
    >
      <Show when={local.icon} fallback={local.children}>
        {(icon) => <Icon class='text-2xl' icon={icon()} />}
      </Show>
    </ButtonPrimitive.Root>
  );
};

export type { IconButtonProps };
export { IconButton, iconButtonVariants };
