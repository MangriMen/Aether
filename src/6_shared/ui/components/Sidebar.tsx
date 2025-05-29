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
      class={cn(local.class, 'h-full flex flex-col items-center pt-2 pb-3')}
      {...others}
    >
      {local.children}
    </div>
  );
};
