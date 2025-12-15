import type { Component, ComponentProps } from 'solid-js';

import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

export type SidebarProps = ComponentProps<'div'>;

export const Sidebar: Component<SidebarProps> = (props) => {
  const [local, others] = splitProps(props as SidebarProps, [
    'class',
    'children',
  ]);

  return (
    <div
      class={cn('flex flex-col items-center px-2 pb-2', local.class)}
      {...others}
    >
      {local.children}
    </div>
  );
};
