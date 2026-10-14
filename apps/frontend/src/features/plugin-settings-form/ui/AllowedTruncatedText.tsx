import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { cn, useIsOverflow } from '@/shared/lib';
import { CombinedTooltip } from '@/shared/ui';

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
