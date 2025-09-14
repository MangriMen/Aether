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
import { CombinedTooltip, IconButton } from '@/shared/ui';

export type AllowedHostProps = {
  changeable?: boolean;
  onEdit?: () => void;
  onRemove?: () => void;
  value?: string;
} & ComponentProps<'div'>;

export const AllowedHost: Component<AllowedHostProps> = (props) => {
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
      <span>{local.value}</span>
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
