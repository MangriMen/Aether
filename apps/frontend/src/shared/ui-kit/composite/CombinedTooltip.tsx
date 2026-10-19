import type { PolymorphicProps } from '@kobalte/core';
import type { ValidComponent, JSX } from 'solid-js';

import { splitProps } from 'solid-js';

import type {
  TooltipTriggerProps,
  TooltipRootProps,
} from '../primitives/Tooltip';

import { Tooltip, TooltipContent, TooltipTrigger } from '../primitives/Tooltip';

export type CombinedTooltipProps<T extends ValidComponent = 'button'> =
  TooltipRootProps &
    TooltipTriggerProps<T> & {
      label?: string | JSX.Element;
      disableTooltip?: boolean;
    };

// Typed against the root props so a Kobalte rename fails compilation instead of leaking into the trigger.
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
  'triggerOnFocusOnly',
  'openDelay',
  'closeDelay',
  'skipDelayDuration',
  'ignoreSafeArea',
  'forceMount',
  'id',
] as const satisfies readonly (keyof TooltipRootProps)[];

export const CombinedTooltip = <T extends ValidComponent = 'button'>(
  props: PolymorphicProps<T, CombinedTooltipProps<T>>,
) => {
  const [local, tooltipRoot, others] = splitProps(
    props,
    ['label', 'disableTooltip'],
    TOOLTIP_ROOT_KEYS,
  );
  return (
    <Tooltip
      {...tooltipRoot}
      disabled={local.disableTooltip}
      // A controlled `open` would otherwise keep the content visible while disabled.
      open={local.disableTooltip ? false : tooltipRoot.open}
    >
      {/* // TODO: find a way to make this work
      //@ts-expect-error no declaration */}
      <TooltipTrigger {...(others as TooltipTriggerProps<T>)} />
      <TooltipContent class='max-w-[90vw]'>{local.label}</TooltipContent>
    </Tooltip>
  );
};
