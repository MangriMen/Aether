import IconMdiArrowRightBold from '~icons/mdi/arrow-right-bold';
import IconMdiClose from '~icons/mdi/close';
import IconMdiMonitor from '~icons/mdi/monitor';
import IconMdiPencil from '~icons/mdi/pencil';
import IconMdiPuzzle from '~icons/mdi/puzzle';
import {
  mergeProps,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedTooltip, IconButton } from '@/shared/ui';

import { AllowedPathComponent } from './AllowedPathComponent';

export type AllowedPathProps = ComponentProps<'div'> & {
  value?: [string, string];
  onEdit?: () => void;
  onRemove?: () => void;
  changeable?: boolean;
};

export const AllowedPath: Component<AllowedPathProps> = (props) => {
  const [_local, others] = splitProps(props, [
    'value',
    'onEdit',
    'onRemove',
    'changeable',
    'class',
  ]);

  const local = mergeProps({ changeable: true }, _local);

  const [{ t }] = useTranslation();

  return (
    <div
      class={cn(
        'flex items-center select-none justify-between gap-2 rounded-sm h-[30px] hover:bg-secondary/50 group px-1',
        local.class,
      )}
      {...others}
    >
      <div class='grid w-full grid-cols-[1fr,auto,1fr] items-center'>
        <AllowedPathComponent
          label={t('pluginSettings.host')}
          icon={IconMdiMonitor}
          value={local.value?.[0]}
        />
        <IconMdiArrowRightBold class='mx-2' />
        <AllowedPathComponent
          label={t('pluginSettings.plugin')}
          icon={IconMdiPuzzle}
          value={local.value?.[1]}
        />
      </div>
      <Show when={local.changeable}>
        <div class='flex items-center opacity-0 transition-opacity group-hover:opacity-100'>
          <CombinedTooltip
            label='Edit'
            as={IconButton}
            class='size-max p-1'
            variant='ghost'
            size='sm'
            icon={IconMdiPencil}
            onClick={local.onEdit}
          />
          <CombinedTooltip
            label='Remove'
            as={IconButton}
            class='size-max p-1'
            variant='ghost'
            size='sm'
            icon={IconMdiClose}
            onClick={local.onRemove}
          />
        </div>
      </Show>
    </div>
  );
};
