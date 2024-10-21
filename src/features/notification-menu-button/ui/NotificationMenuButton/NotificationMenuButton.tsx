import { Icon } from '@iconify-icon/solid';
import {
  Component,
  createEffect,
  createSignal,
  For,
  onMount,
  Show,
} from 'solid-js';

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
  listenLoading,
  LoadingBarTypeEnum,
  LoadingPayload,
} from '@/entities/minecraft';

import { NotificationMenuButtonProps } from './types';

export const NotificationMenuButton: Component<NotificationMenuButtonProps> = (
  props,
) => {
  const [events, setEvents] = createSignal<Record<string, LoadingPayload>>({});

  const [isMenuOpen, setIsMenuOpen] = createSignal(false);
  const [isNewEvent, setIsNewEvent] = createSignal(false);

  const fetchEvents = async () => {
    try {
      const bars = await getLoadingBars();

      Object.entries(bars).forEach(([_, value]) => {
        console.log(value);
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

  const addEvent = (e: LoadingPayload) => {
    if (e.fraction === null) {
      setEvents((prev) => {
        const newEvents = { ...prev };
        delete newEvents[e.loaderUuid];
        return newEvents;
      });
      return;
    }

    setEvents((prev) => ({ ...prev, [e.loaderUuid]: e }));
  };

  onMount(() => {
    fetchEvents();

    listenLoading((e) => {
      if ((e.payload.fraction ?? 1) <= 0.05) {
        setIsNewEvent(true);
      }

      if (
        e.payload.fraction === undefined ||
        e.payload.message === 'Completed'
      ) {
        refetchInstances();
      }

      addEvent(e.payload);
    });
  });

  createEffect(() => {
    if (isNewEvent() && !isMenuOpen() && Object.keys(events()).length > 0) {
      setIsMenuOpen(true);
      setIsNewEvent(false);
    }
  });

  return (
    <Popover open={isMenuOpen()} onOpenChange={setIsMenuOpen}>
      <Show when={Object.keys(events()).length}>
        <PopoverTrigger as={IconButton} variant='ghost' {...props}>
          <Icon class='text-2xl' icon='mdi-menu-down' />
        </PopoverTrigger>
      </Show>
      <PopoverContent class='flex flex-col gap-3 border-none bg-transparent p-0 pt-2'>
        <For each={Object.values(events())}>
          {(payload) => <EventCard payload={payload} />}
        </For>
      </PopoverContent>
    </Popover>
  );
};
