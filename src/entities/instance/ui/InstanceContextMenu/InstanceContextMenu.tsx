import { Component, splitProps } from 'solid-js';

import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
} from '@/shared/ui';

import { InstanceContextMenuProps } from './types';

export const InstanceContextMenu: Component<InstanceContextMenuProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'onPlay',
    'onRemove',
    'isLoading',
    'children',
  ]);

  return (
    <ContextMenu {...others}>
      {local.children}
      <ContextMenuContent>
        <ContextMenuItem
          class='w-full hover:!bg-success hover:text-success-foreground'
          onClick={local.onPlay}
          disabled={local.isLoading}
        >
          Play
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem
          class='w-full hover:!bg-destructive hover:text-destructive-foreground'
          onClick={local.onRemove}
          disabled={local.isLoading}
        >
          Delete
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
};
