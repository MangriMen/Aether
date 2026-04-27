import IconMdiChevronDown from '~icons/mdi/chevron-down';
import { splitProps, type Component } from 'solid-js';

import { cn } from '@/shared/lib';

import { IconButton, type IconButtonProps } from './IconButton';

export type CollapsibleIconProps = IconButtonProps & {
  open?: boolean;
};

export const CollapsibleIcon: Component<CollapsibleIconProps> = (props) => {
  const [local, others] = splitProps(props, ['open', 'class']);

  return (
    <IconButton
      variant={null}
      icon={() => (
        <IconMdiChevronDown
          class={cn('rotate-0 transition-transform', {
            '-rotate-180': local.open,
          })}
        />
      )}
      {...others}
    />
  );
};
