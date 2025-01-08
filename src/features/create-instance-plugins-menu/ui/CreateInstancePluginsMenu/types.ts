import { DialogRootProps } from '@kobalte/core/dialog';
import { ComponentProps } from 'solid-js';

export type CreateInstancePluginsMenuProps = ComponentProps<'div'> &
  Pick<DialogRootProps, 'onOpenChange'>;
