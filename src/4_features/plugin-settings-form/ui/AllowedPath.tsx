import IconMdiArrowRightBold from '~icons/mdi/arrow-right-bold';
import IconMdiMonitor from '~icons/mdi/monitor';
import IconMdiPuzzle from '~icons/mdi/puzzle';
import {
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import type { AllowedItemProps } from '../model';

import { AllowedPathComponent } from './AllowedPathComponent';

export type AllowedPathProps = ComponentProps<'div'> &
  AllowedItemProps<[string, string], [string, string]>;

export const AllowedPath: Component<AllowedPathProps> = (props) => {
  const [local, others] = splitProps(props, [
    'value',
    'error',
    'leadingItems',
    'class',
  ]);

  const [{ t }] = useTranslation();

  return (
    <div
      class={cn(
        'group flex min-h-[30px] select-none items-start justify-between gap-2 rounded-sm py-1 px-2 hover:bg-secondary/50',
        local.class,
      )}
      {...others}
    >
      <div class='grid w-full grid-cols-[1fr,auto,1fr] items-start'>
        <div class='flex flex-col'>
          <AllowedPathComponent
            label={t('pluginSettings.host')}
            icon={IconMdiMonitor}
            value={local.value?.[0]}
          />
          <span class='text-destructive'>{local.error?.[0]}</span>
        </div>
        <IconMdiArrowRightBold class='mx-2 mt-[3px]' />
        <div class='flex flex-col'>
          <AllowedPathComponent
            label={t('pluginSettings.plugin')}
            icon={IconMdiPuzzle}
            value={local.value?.[1]}
          />
          <span class='text-destructive'>{local.error?.[1]}</span>
        </div>
      </div>
      <Show when={local.leadingItems}>
        {(leadingItems) => (
          <div class='flex items-center opacity-0 transition-opacity group-hover:opacity-100'>
            {leadingItems()}
          </div>
        )}
      </Show>
    </div>
  );
};
