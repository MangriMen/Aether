import IconMdiClose from '~icons/mdi/close';
import IconMdiPencil from '~icons/mdi/pencil';
import {
  mergeProps,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { cn } from '@/shared/lib';
import { CombinedTooltip, IconButton, Separator } from '@/shared/ui';

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

  return (
    <div
      class={cn(
        'flex items-center select-none justify-between gap-2 rounded-sm h-[30px] hover:bg-secondary/50 group',
        local.class,
      )}
      {...others}
    >
      <div class='grid w-full grid-cols-[1fr,auto,1fr]'>
        <span>{local.value?.[0] ?? ''}</span>
        <Separator class='w-0.5 bg-primary' orientation='vertical' />
        <span class='pl-2'>{local.value?.[1] ?? ''}</span>
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
