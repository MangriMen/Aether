import type { ContextMenuRootProps } from '@kobalte/core/context-menu';
import type { Component } from 'solid-js';

import { splitProps } from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
} from '@/shared/ui-kit';

export type BrokenInstanceContextMenuProps = ContextMenuRootProps & {
  onOpenFolder?: () => void;
  onRemove?: () => void;
  disableOpenFolder?: boolean;
};

/** The menu of a damaged instance: only what can be done with one. */
export const BrokenInstanceContextMenu: Component<
  BrokenInstanceContextMenuProps
> = (props) => {
  const [local, others] = splitProps(props, [
    'onOpenFolder',
    'onRemove',
    'disableOpenFolder',
    'children',
  ]);

  const [{ t }] = useTranslation();

  return (
    <ContextMenu {...others}>
      {local.children}
      <ContextMenuContent>
        <ContextMenuItem
          class='w-full'
          onSelect={local.onOpenFolder}
          disabled={local.disableOpenFolder}
        >
          {t('instance.openFolder')}
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem
          class='
            hover:bg-destructive! hover:text-destructive-foreground
            w-full
          '
          onSelect={local.onRemove}
        >
          {t('common.remove')}
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
};
