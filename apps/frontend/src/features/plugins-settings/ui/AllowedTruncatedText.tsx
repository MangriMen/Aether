import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { useIsOverflow, CombinedTooltip } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

export type AllowedTruncatedTextProps = ComponentProps<'span'> & {
  value?: string;
  tooltipPrefix?: string;
};

export const AllowedTruncatedText: Component<AllowedTruncatedTextProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'value',
    'tooltipPrefix',
    'class',
  ]);

  const [isOverflowed, ref, updateOverflow] = useIsOverflow();

  return (
    <CombinedTooltip
      as='span'
      ref={ref}
      label={
        local.tooltipPrefix
          ? `${local.tooltipPrefix}: ${local.value ?? ''}`
          : local.value
      }
      onPointerEnter={updateOverflow}
      disableTooltip={!isOverflowed()}
      class={cn('min-w-0 truncate', local.class)}
      {...others}
    >
      {local.value}
    </CombinedTooltip>
  );
};
