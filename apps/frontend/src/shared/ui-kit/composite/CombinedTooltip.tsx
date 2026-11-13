import type { PolymorphicProps } from '@kobalte/core';
import type { ValidComponent, JSX } from 'solid-js';

import { children, splitProps } from 'solid-js';

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
  const [content, rest] = splitProps(others, ['children']);
  const resolved = children(() => content.children);
  // A trigger with its own text already has a name; overriding it would hide that text from screen readers.
  const hasText = () =>
    resolved
      .toArray()
      .some((node) => typeof node === 'string' || typeof node === 'number');
  return (
    <Tooltip
      {...tooltipRoot}
      disabled={local.disableTooltip}
      // A controlled `open` would otherwise keep the content visible while disabled.
      open={local.disableTooltip ? false : tooltipRoot.open}
    >
      {/* // TODO: find a way to make this work
      //@ts-expect-error no declaration */}
      <TooltipTrigger
        // Kobalte sets aria-describedby only while the tooltip is open, so an icon-only trigger needs its own name.
        aria-label={
          typeof local.label === 'string' && !hasText()
            ? local.label
            : undefined
        }
        {...(rest as TooltipTriggerProps<T>)}
      >
        {resolved()}
      </TooltipTrigger>
      <TooltipContent class='max-w-[90vw]'>{local.label}</TooltipContent>
    </Tooltip>
  );
};
