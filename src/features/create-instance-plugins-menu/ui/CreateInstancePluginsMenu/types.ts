import type { DialogRootProps } from '@kobalte/core/dialog';
import type { ComponentProps } from 'solid-js';

export type CreateInstancePluginsMenuProps = ComponentProps<'div'> &
  Pick<DialogRootProps, 'onOpenChange'>;
