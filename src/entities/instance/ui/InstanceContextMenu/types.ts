import { ContextMenuRootProps } from '@kobalte/core/context-menu';

export type InstanceContextMenuProps = ContextMenuRootProps & {
  onPlay?: () => void;
  onRemove?: () => void;
  isLoading?: boolean;
};
