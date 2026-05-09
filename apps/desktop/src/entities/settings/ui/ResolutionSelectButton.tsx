import type { PhysicalSize } from '@tauri-apps/api/dpi';
import type { Component } from 'solid-js';

import { currentMonitor } from '@tauri-apps/api/window';
import IconMdiMonitor from '~icons/mdi/monitor-screenshot';
import {
  createMemo,
  createSignal,
  For,
  onMount,
  Show,
  splitProps,
} from 'solid-js';

import {
  Badge,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  IconButton,
} from '@/shared/ui';

import { RESOLUTION_OPTIONS } from '../model';

export type ResolutionSelectButtonProps = {
  disabled?: boolean;
  onResolutionChange?: ([width, height]: [number, number]) => void;
};

export const ResolutionSelectButton: Component<ResolutionSelectButtonProps> = (
  props,
) => {
  const [local, _] = splitProps(props, ['disabled', 'onResolutionChange']);

  const [maxResolution, setMaxResolution] = createSignal<
    PhysicalSize | undefined
  >();

  const updateMaxResolution = async () => {
    const monitor = await currentMonitor();
    setMaxResolution(monitor?.size);
  };

  const handleDropdownOpenChange = (isOpen: boolean) => {
    if (isOpen) {
      updateMaxResolution();
    }
  };

  const filteredResolutions = createMemo(() => {
    const max = maxResolution();

    if (max === undefined) {
      return RESOLUTION_OPTIONS;
    }

    return RESOLUTION_OPTIONS.filter(({ value: [width, height] }) => {
      return width <= max.width && height <= max.height;
    });
  });

  const isDefaultResolution = ([width, height]: [number, number]) => {
    return width === 960 && height === 540;
  };

  onMount(() => {
    updateMaxResolution();
  });

  return (
    <DropdownMenu onOpenChange={handleDropdownOpenChange}>
      <DropdownMenuTrigger
        as={IconButton}
        class='flex'
        variant='secondary'
        size='sm'
        disabled={local.disabled}
        icon={IconMdiMonitor}
      />
      <DropdownMenuContent class='max-h-[230px] overflow-auto'>
        <For each={filteredResolutions()}>
          {(option) => (
            <DropdownMenuItem
              onClick={() => local.onResolutionChange?.(option.value)}
            >
              {option.name}
              <Show when={isDefaultResolution(option.value)}>
                <Badge variant='secondary'>default</Badge>
              </Show>
            </DropdownMenuItem>
          )}
        </For>
      </DropdownMenuContent>
    </DropdownMenu>
  );
};
