import { Component, createMemo, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

import { Button, ButtonProps } from '../Button';

import { IconButtonProps } from '.';

const getIconColor = (variant: ButtonProps['variant']) => {
  switch (variant) {
    case 'default':
    case 'secondary':
      return '[&_svg]:fill-background';
    case 'outline':
    case 'ghost':
    case 'link':
      return '[&_svg]:fill-foreground';
  }
};

export const IconButton: Component<IconButtonProps> = (props) => {
  const [local, others] = splitProps(props as IconButtonProps, ['variant']);

  const iconColorClass = createMemo(() => getIconColor(local.variant));

  return (
    <Button
      class={cn('p-0.5 rounded-full', iconColorClass())}
      variant={local.variant}
      size='icon'
      {...others}
    />
  );
};
