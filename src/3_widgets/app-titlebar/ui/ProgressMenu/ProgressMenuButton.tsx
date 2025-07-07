import type { Component, ValidComponent } from 'solid-js';
import { createMemo, createSignal, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import type { IconButtonProps } from '@/shared/ui';
import {
  IconButton,
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/shared/ui';

import type { PolymorphicProps } from '@kobalte/core';
import type { ProgressPopover } from './ProgressPopover';
import { useProgressStore } from '../../model';
import { ProgressBadge } from './ProgressBadge';
import { useProgressMenuShowActions } from '../../lib';

export type ProgressMenuButtonProps<T extends ValidComponent = 'button'> =
  PolymorphicProps<T, IconButtonProps<T>> & {
    popoverComponent: typeof ProgressPopover;
  };

export const ProgressMenuButton: Component<ProgressMenuButtonProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['popoverComponent', 'class']);

  const [progressStore] = useProgressStore();
  const payloadValues = createMemo(() => Object.values(progressStore.payloads));

  const [isOpen, setIsOpen] = createSignal(false);
  useProgressMenuShowActions(
    () => payloadValues().length,
    () => setIsOpen(false),
    () => setIsOpen(true),
  );

  return (
    <Popover open={isOpen()} onOpenChange={setIsOpen}>
      <PopoverTrigger
        as={IconButton}
        class={cn('aspect-square p-0 px-1', local.class, {
          'bg-success/10': isOpen(),
          invisible: !payloadValues().length,
        })}
        variant='ghost'
        {...others}
      >
        <ProgressBadge />
      </PopoverTrigger>
      <PopoverContent
        class='bg-background/20 p-3'
        as={local.popoverComponent}
        payloads={payloadValues}
      />
    </Popover>
  );
};
