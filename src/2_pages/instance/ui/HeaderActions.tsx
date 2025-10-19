import type { RowModel } from '@tanstack/solid-table';

import IconMdiReload from '~icons/mdi/reload';
import { Show, type Component } from 'solid-js';

import type { Instance, InstanceFile } from '@/entities/instances';

import { useTranslation } from '@/shared/model';
import { Button } from '@/shared/ui';

import { SelectedRowsActions } from './SelectedRowsActions';

export type HeaderActionsProps = {
  instanceId: Instance['id'];
  allRowsSelected?: boolean;
  someRowsSelected?: boolean;
  refetch?: () => void;
  isLoading?: boolean;
  selectedRows: RowModel<InstanceFile>;
};

export const HeaderActions: Component<HeaderActionsProps> = (props) => {
  const [{ t }] = useTranslation();

  return (
    <Show
      when={props.allRowsSelected || props.someRowsSelected}
      fallback={
        <Show when={props.refetch}>
          <Button
            variant='ghost'
            size='sm'
            leadingIcon={IconMdiReload}
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
