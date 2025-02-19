import { Show, type Component } from 'solid-js';
import MdiReloadIcon from '@iconify/icons-mdi/reload';
import { Button } from '@/shared/ui';
import { SelectedRowsActions } from './SelectedRowsActions';

export type HeaderActionsProps = {
  allRowsSelected?: boolean;
  someRowsSelected?: boolean;
  refetch?: () => void;
  isLoading?: boolean;
};

export const HeaderActions: Component<HeaderActionsProps> = (props) => {
  return (
    <Show
      when={props.allRowsSelected || props.someRowsSelected}
      fallback={
        <Show when={props.refetch}>
          <Button
            variant='ghost'
            size='sm'
            leadingIcon={MdiReloadIcon}
            loading={props.isLoading}
            onClick={props.refetch}
            children='Refresh'
          />
        </Show>
      }
    >
      <SelectedRowsActions />
    </Show>
  );
};
