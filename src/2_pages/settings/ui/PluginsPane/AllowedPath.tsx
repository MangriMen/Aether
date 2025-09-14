import MdiCloseIcon from '@iconify/icons-mdi/close';
import MdiPencilIcon from '@iconify/icons-mdi/pencil';
import {
  type Component,
  type ComponentProps,
  mergeProps,
  Show,
  splitProps,
} from 'solid-js';

import { cn } from '@/shared/lib';
import { CombinedTooltip, IconButton, Separator } from '@/shared/ui';

export type AllowedPathProps = {
  changeable?: boolean;
  onEdit?: () => void;
  onRemove?: () => void;
  value?: [string, string];
} & ComponentProps<'div'>;

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
            as={IconButton}
            class='size-max p-1'
            icon={MdiPencilIcon}
            label='Edit'
            onClick={local.onEdit}
            size='sm'
            variant='ghost'
          />
          <CombinedTooltip
            as={IconButton}
            class='size-max p-1'
            icon={MdiCloseIcon}
            label='Remove'
            onClick={local.onRemove}
            size='sm'
            variant='ghost'
          />
        </div>
      </Show>
    </div>
  );
};
