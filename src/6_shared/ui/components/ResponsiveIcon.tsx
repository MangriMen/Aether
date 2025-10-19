import type { Component, ComponentProps } from 'solid-js';

import { createElementSize } from '@solid-primitives/resize-observer';
import { createMemo, createSignal, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

export type ResponsiveIconProps = {
  icon: Component<ComponentProps<'svg'>>;
  class?: string;
};

export const ResponsiveIcon: Component<ResponsiveIconProps> = (props) => {
  const [local, others] = splitProps(props, ['icon', 'class']);
  const [target, setTarget] = createSignal<HTMLElement>();

  const size = createElementSize(target);

  const iconSize = createMemo(() =>
    size.height ? `${size.height}px` : undefined,
  );

  const style = createMemo(() => ({
    'font-size': iconSize(),
  }));

  return (
    <local.icon
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
