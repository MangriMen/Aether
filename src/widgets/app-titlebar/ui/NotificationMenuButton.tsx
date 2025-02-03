import MdiMenuDownIcon from '@iconify/icons-mdi/menu-down';
import { Icon } from '@iconify-icon/solid';
import { ReactiveMap } from '@solid-primitives/map';
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
import { refetchInstances } from '@/entities/instances';
import type { LoadingPayload } from '@/entities/events';
import type { PolymorphicProps } from '@kobalte/core';

const NOTIFICATION_COMPLETED_REMOVE_DELAY = 1500;

export type NotificationMenuButtonProps<T extends ValidComponent = 'button'> =
  PolymorphicProps<T, IconButtonProps<T>>;

export const NotificationMenuButton: Component<NotificationMenuButtonProps> = (
  props,
) => {
  const events = new ReactiveMap<string, LoadingPayload>();

  const [isMenuOpen, setIsMenuOpen] = createSignal(false);
  const [isNewEvent, setIsNewEvent] = createSignal(false);

  const [removeNotificationTimer, setRemoveNotificationTimer] = createSignal<
    number | undefined
  >(undefined);

  const fetchEvents = async () => {
    try {
      const bars = await getLoadingBars();

      Object.entries(bars).forEach(([_, value]) => {
        addEvent({
          event: value.barType,
          loaderUuid: value.loadingBarUuid,
          message: value.message,
          fraction: value.current / value.total,
        });
      });
    } catch {
      /* empty */
    }
  };

  const removeEvent = (payload: LoadingPayload) => {
    setRemoveNotificationTimer(
      setTimeout(() => {
        events.delete(payload.loaderUuid);
      }, NOTIFICATION_COMPLETED_REMOVE_DELAY),
    );
  };

  const addEvent = (payload: LoadingPayload) => {
    events.set(payload.loaderUuid, payload);
  };

  const listenEvents = () => {
    listenEvent<LoadingPayload>('loading', (e) => {
      if (isDebug()) {
        console.log('[EVENT][DEBUG]', e);
      }

      if ((e.payload.fraction ?? 1) <= 0.05) {
        setIsNewEvent(true);
        refetchInstances();
      }

      if (e.payload.fraction === null) {
        removeEvent(e.payload);
        refetchInstances();
      } else {
        addEvent(e.payload);
      }
    });
  };

  const payloads = createMemo(() => [...events.values()]);

  onMount(() => {
    fetchEvents();
    listenEvents();
  });

  createEffect(() => {
    if (isNewEvent() && !isMenuOpen() && events.size > 0) {
      setIsMenuOpen(true);
      setIsNewEvent(false);
    }
  });

  onCleanup(() => {
    clearTimeout(removeNotificationTimer());
    setRemoveNotificationTimer(undefined);
  });

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
