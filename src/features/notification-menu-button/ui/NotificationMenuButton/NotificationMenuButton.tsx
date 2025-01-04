import { Icon } from '@iconify-icon/solid';
import { ReactiveMap } from '@solid-primitives/map';
import {
  Component,
  createEffect,
  createMemo,
  createSignal,
  For,
  onCleanup,
  onMount,
  Show,
} from 'solid-js';

import { isDebug } from '@/shared/model/settings';
import {
  IconButton,
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/shared/ui';

import { EventCard } from '@/entities/events';
import { refetchInstances } from '@/entities/instance';
import {
  getLoadingBars,
  listenEvent,
  LoadingBarTypeEnum,
  LoadingPayload,
} from '@/entities/minecraft';

import { NotificationMenuButtonProps } from './types';

const NOTIFICATION_COMPLETED_REMOVE_DELAY = 1500;

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
        if (value.barType.type === LoadingBarTypeEnum.MinecraftDownload) {
          addEvent({
            event: value.barType,
            loaderUuid: value.loadingBarUuid,
            message: value.message,
            fraction: value.current / value.total,
          });
        }
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
    listenEvent('loading', (e) => {
      if (isDebug()) {
        console.log('[EVENT][DEBUG]', e);
      }

      if ((e.payload.fraction ?? 1) <= 0.05) {
        setIsNewEvent(true);
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
        <PopoverTrigger as={IconButton} variant='ghost' {...props}>
          <Icon class='text-2xl' icon='mdi-menu-down' />
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
