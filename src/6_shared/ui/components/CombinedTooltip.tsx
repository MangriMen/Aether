import type { PolymorphicProps } from '@kobalte/core';
import type { ValidComponent, JSX } from 'solid-js';

import { splitProps } from 'solid-js';

import type { TooltipTriggerProps, TooltipRootProps } from './Tooltip';

import { Tooltip, TooltipContent, TooltipTrigger } from './Tooltip';

export type CombinedTooltipProps<T extends ValidComponent = 'button'> =
  TooltipRootProps &
    TooltipTriggerProps<T> & {
      label?: string | JSX.Element;
      disableTooltip?: boolean;
    };

const TOOLTIP_ROOT_KEYS = [
  'getAnchorRect',
  'placement',
  'gutter',
  'shift',
  'flip',
  'slide',
  'overlap',
  'sameWidth',
  'fitViewport',
  'hideWhenDetached',
  'detachedPadding',
  'arrowPadding',
  'overflowPadding',
  'open',
  'defaultOpen',
  'onOpenChange',
  'disableTooltip',
  'triggerOnFocusOnly',
  'openDelay',
  'closeDelay',
  'skipDelayDuration',
  'ignoreSafeArea',
  'forceMount',
  'id',
] as const;

export const CombinedTooltip = <T extends ValidComponent = 'button'>(
  props: PolymorphicProps<T, CombinedTooltipProps<T>>,
) => {
  const [local, tooltipRoot, others] = splitProps(
    props,
    ['label', 'disableTooltip'],
    TOOLTIP_ROOT_KEYS,
  );
  return (
    <Tooltip disabled={local.disableTooltip} {...tooltipRoot}>
      {/* // TODO: find a way to make this work
      //@ts-expect-error no declaration */}
      <TooltipTrigger {...(others as TooltipTriggerProps<T>)} />
      <TooltipContent>{local.label}</TooltipContent>
    </Tooltip>
  );
};
