import { cn } from '@/shared/lib';
import { CombinedTooltip, IconButton } from '@/shared/ui';
import {
  mergeProps,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';
import MdiPencilIcon from '@iconify/icons-mdi/pencil';
import MdiCloseIcon from '@iconify/icons-mdi/close';

export type AllowedHostProps = ComponentProps<'div'> & {
  value?: string;
  onEdit?: () => void;
  onRemove?: () => void;
  changeable?: boolean;
};

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
            label='Edit'
            as={IconButton}
            class='size-max p-1'
            variant='ghost'
            size='sm'
            icon={MdiPencilIcon}
            onClick={local.onEdit}
          />
          <CombinedTooltip
            label='Remove'
            as={IconButton}
            class='size-max p-1'
            variant='ghost'
            size='sm'
            icon={MdiCloseIcon}
            onClick={local.onRemove}
          />
        </div>
      </Show>
    </div>
  );
};
