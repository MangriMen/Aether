import type { Component } from 'solid-js';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

import type { SidebarProps } from '.';

export const Sidebar: Component<SidebarProps> = (props) => {
  const [local, others] = splitProps(props as SidebarProps, [
    'class',
    'children',
  ]);

  return (
    <div
      class={cn(
        local.class,
        'h-full bg-secondary-dark flex flex-col items-center pt-2 pb-3',
      )}
      {...others}
    >
      {local.children}
    </div>
  );
};
