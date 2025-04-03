import { Show, type Component } from 'solid-js';
import MdiReloadIcon from '@iconify/icons-mdi/reload';
import { Button } from '@/shared/ui';
import { SelectedRowsActions } from './SelectedRowsActions';
import { useTranslate } from '@/shared/model';
import type { RowModel } from '@tanstack/solid-table';
import type { Instance, InstanceFile } from '@/entities/instances';

export type HeaderActionsProps = {
  instanceId: Instance['id'];
  allRowsSelected?: boolean;
  someRowsSelected?: boolean;
  refetch?: () => void;
  isLoading?: boolean;
  selectedRows: RowModel<InstanceFile>;
};

export const HeaderActions: Component<HeaderActionsProps> = (props) => {
  const [{ t }] = useTranslate();

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
            children={t('common.refresh')}
          />
        </Show>
      }
    >
      <SelectedRowsActions
        instanceId={props.instanceId}
        selectedRows={props.selectedRows}
      />
    </Show>
  );
};
