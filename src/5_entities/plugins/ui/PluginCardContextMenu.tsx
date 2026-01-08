import type { ContextMenuRootProps } from '@kobalte/core/context-menu';
import type { Component, ComponentProps } from 'solid-js';

import { splitProps, Switch, Match } from 'solid-js';

import { stopPropagation } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
} from '@/shared/ui';

export type PluginContextMenuProps = ContextMenuRootProps & {
  onToggleEnabled?: ComponentProps<'button'>['onClick'];
  onRemove?: ComponentProps<'button'>['onClick'];
  isDisabled?: boolean;
  isLoading?: boolean;
  isEnabled?: boolean;
};

export const PluginContextMenu: Component<PluginContextMenuProps> = (props) => {
  const [local, others] = splitProps(props, [
    'onToggleEnabled',
    'onRemove',
    'isDisabled',
    'isLoading',
    'isEnabled',
    'children',
  ]);

  const [{ t }] = useTranslation();

  return (
    <ContextMenu {...others}>
      {local.children}
      <ContextMenuContent
        onClick={stopPropagation}
        onMouseDown={stopPropagation}
        onKeyDown={stopPropagation}
      >
        <ContextMenuItem
          class='w-full hover:!bg-primary hover:text-primary-foreground'
          onClick={local.onToggleEnabled}
          disabled={local.isLoading}
        >
          <Switch>
            <Match when={local.isDisabled || local.isLoading}>
              {t('common.enable')}
            </Match>
            <Match when={local.isEnabled}>{t('common.disable')}</Match>
          </Switch>
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem
          class='w-full hover:!bg-destructive hover:text-destructive-foreground'
          onClick={local.onRemove}
          disabled={local.isLoading}
        >
          {t('common.remove')}
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
};
