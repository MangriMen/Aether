import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { CombinedTooltip } from '@/shared/ui';

export type AllowedPathComponentProps = ComponentProps<'div'> & {
  value?: string;
  icon?: Component;
  label?: string;
};

export const AllowedPathComponent: Component<AllowedPathComponentProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['value', 'icon', 'class']);

  return (
    <CombinedTooltip
      as='div'
      class={cn('flex items-center gap-2', local.class)}
      {...others}
    >
      {local.icon?.({})}
      <span class='grow'>{local.value}</span>
    </CombinedTooltip>
  );
};
