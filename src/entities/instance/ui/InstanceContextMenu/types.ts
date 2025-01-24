import { ContextMenuRootProps } from '@kobalte/core/context-menu';
import { ComponentProps } from 'solid-js';

export type InstanceContextMenuProps = ContextMenuRootProps & {
  onPlay?: ComponentProps<'button'>['onClick'];
  onOpenFolder?: ComponentProps<'button'>['onClick'];
  onRemove?: ComponentProps<'button'>['onClick'];
  isLoading?: boolean;
};
