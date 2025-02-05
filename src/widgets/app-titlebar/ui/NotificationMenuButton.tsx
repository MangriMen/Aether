import MdiMenuDownIcon from '@iconify/icons-mdi/menu-down';
import { Icon } from '@iconify-icon/solid';
import type { Component, ValidComponent } from 'solid-js';
import {
  createEffect,
  createMemo,
  createSignal,
  For,
  onCleanup,
  onMount,
  Show,
} from 'solid-js';

import { cn } from '@/shared/lib';
import { isDebug } from '@/shared/model';
import type { IconButtonProps } from '@/shared/ui';
import {
  IconButton,
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/shared/ui';

import { EventCard, getLoadingBars, listenEvent } from '@/entities/events';
import type { LoadingPayload } from '@/entities/events';
import type { PolymorphicProps } from '@kobalte/core';
import { COMPLETED_NOTIFICATION_REMOVE_DELAY } from '../model/constants';
import { ReactiveMap } from '@solid-primitives/map';
import type { UnlistenFn } from '@tauri-apps/api/event';

export type NotificationMenuButtonProps<T extends ValidComponent = 'button'> =
  PolymorphicProps<T, IconButtonProps<T>>;

export const NotificationMenuButton: Component<NotificationMenuButtonProps> = (
  props,
) => {
  const displayEventsMap = new ReactiveMap<string, LoadingPayload>();

  const [isMenuOpen, setIsMenuOpen] = createSignal(false);
  const [isNewEvent, setIsNewEvent] = createSignal(false);

  let removeNotificationTimer: number | undefined;

  const resetTimer = () => {
    clearTimeout(removeNotificationTimer);
    removeNotificationTimer = undefined;
  };

  const addEvent = (payload: LoadingPayload) => {
    displayEventsMap.set(payload.loaderUuid, payload);
  };

  const removeEvent = (payload: LoadingPayload) => {
    removeNotificationTimer = setTimeout(
      () => displayEventsMap.delete(payload.loaderUuid),
      COMPLETED_NOTIFICATION_REMOVE_DELAY,
    );
  };

  const fetchEvents = async () => {
    try {
      const bars = await getLoadingBars();

      for (const bar of Object.values(bars)) {
        addEvent({
          event: bar.barType,
          loaderUuid: bar.loadingBarUuid,
          message: bar.message,
          fraction: bar.current / bar.total,
        });
      }
    } catch {
      /* empty */
    }
  };

  let loadingEventsUnlistenFn: UnlistenFn | undefined;

  const startLoadingListener = () =>
    listenEvent('loading', (e) => {
      if (isDebug()) {
        console.log('[EVENT][DEBUG]', e);
      }

      if (e.payload.fraction === null) {
        removeEvent(e.payload);
        return;
      }

      if (e.payload.fraction <= 0.05) {
        setIsNewEvent(true);
      }

      addEvent(e.payload);
    });

  const stopLoadingListener = () => loadingEventsUnlistenFn?.();

  const payloads = createMemo(() => [...displayEventsMap.values()]);

  createEffect(() => {
    if (isNewEvent() && !isMenuOpen() && displayEventsMap.size > 0) {
      setIsMenuOpen(true);
      setIsNewEvent(false);
    }
  });

  onMount(() => {
    fetchEvents();
    startLoadingListener().then((unlistenFn) => {
      loadingEventsUnlistenFn = unlistenFn;
    });
  });

  onCleanup(resetTimer);
  onCleanup(stopLoadingListener);

  return (
    <Popover open={isMenuOpen()} onOpenChange={setIsMenuOpen}>
      <Show when={payloads().length}>
        <PopoverTrigger
          as={IconButton}
          class='aspect-square p-0'
          variant='ghost'
          {...props}
        >
          <Icon
            class={cn('text-2xl transition-transform duration-300', {
              'rotate-180': isMenuOpen(),
            })}
            icon={MdiMenuDownIcon}
          />
        </PopoverTrigger>
      </Show>
      <PopoverContent class='flex flex-col gap-3 border-none bg-transparent p-0 pt-2'>
        <For each={payloads()}>
          {(payload) => <EventCard payload={payload} />}
        </For>
      </PopoverContent>
    </Popover>
  );
};
