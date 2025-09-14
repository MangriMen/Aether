import type { RowModel } from '@tanstack/solid-table';

import MdiReloadIcon from '@iconify/icons-mdi/reload';
import { type Component, Show } from 'solid-js';

import type { Instance, InstanceFile } from '@/entities/instances';

import { useTranslation } from '@/shared/model';
import { Button } from '@/shared/ui';

import { SelectedRowsActions } from './SelectedRowsActions';

export type HeaderActionsProps = {
  allRowsSelected?: boolean;
  instanceId: Instance['id'];
  isLoading?: boolean;
  refetch?: () => void;
  selectedRows: RowModel<InstanceFile>;
  someRowsSelected?: boolean;
};

export const HeaderActions: Component<HeaderActionsProps> = (props) => {
  const [{ t }] = useTranslation();

  return (
    <Show
      fallback={
        <Show when={props.refetch}>
          <Button
            children={t('common.refresh')}
            leadingIcon={MdiReloadIcon}
            loading={props.isLoading}
            onClick={props.refetch}
            size='sm'
            variant='ghost'
          />
        </Show>
      }
      when={props.allRowsSelected || props.someRowsSelected}
    >
      <SelectedRowsActions
        instanceId={props.instanceId}
        selectedRows={props.selectedRows}
      />
    </Show>
  );
};
