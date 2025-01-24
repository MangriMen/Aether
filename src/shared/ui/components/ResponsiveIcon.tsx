import { Icon, IconifyIconProps } from '@iconify-icon/solid';
import { createElementSize } from '@solid-primitives/resize-observer';
import { Component, createMemo, createSignal, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

export type ResponsiveIconProps = IconifyIconProps;

export const ResponsiveIcon: Component<ResponsiveIconProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);
  const [target, setTarget] = createSignal<HTMLElement>();

  const size = createElementSize(target);

  const iconSize = createMemo(() =>
    size.height ? `${size.height}px` : undefined,
  );

  const style = createMemo(() => ({
    'font-size': iconSize(),
  }));

  return (
    <Icon
      ref={setTarget}
      class={cn(
        'inline-flex size-full items-center justify-center',
        local.class,
      )}
      style={style()}
      {...others}
    />
  );
};
