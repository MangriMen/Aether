import type { DialogRootProps } from '@kobalte/core/dialog';
import type { ComponentProps } from 'solid-js';

export const TABS = {
  Custom: 'custom',
  Import: 'import',
} as const;

export const TAB_VALUES = Object.values(TABS);

export type TabKey = (typeof TABS)[keyof typeof TABS];

export type TabContentProps = ComponentProps<'div'> &
  Pick<DialogRootProps, 'onOpenChange'>;
