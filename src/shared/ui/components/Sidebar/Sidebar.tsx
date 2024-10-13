import { Component, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

import { SidebarProps } from '.';

export const Sidebar: Component<SidebarProps> = (props) => {
  const [local, others] = splitProps(props as SidebarProps, [
    'class',
    'children',
  ]);

  return (
    <div
      class={cn(
        local.class,
        'h-full min-w-16 max-w-16 bg-secondary-dark flex flex-col items-center py-2',
      )}
      {...others}
    >
      {local.children}
    </div>
  );
};
