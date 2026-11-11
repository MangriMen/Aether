import type { Component } from 'solid-js';

import { createEffect } from 'solid-js';

import { type ContentFile } from '@/features/content';
import { useTranslation } from '@/shared/i18n';
import { CombinedPagination, DataTable } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import { createContentTable } from '../lib/createContentTable';
import { ContentActions } from './ContentActions';
import { HeaderActions } from './HeaderActions';
import { SelectionToolbar } from './SelectionToolbar';

export type ContentTableProps = {
  data: ContentFile[];
  searchQuery?: string;
  isLoading?: boolean;
  refetch?: () => void;
  instanceId: string;
  instancePath?: string;
};

export const ContentTable: Component<ContentTableProps> = (props) => {
  const [{ t }] = useTranslation();

  const table = createContentTable({
    headerActions: (localProps) => <HeaderActions {...localProps} />,
    contentActions: (localProps) => (
      <ContentActions
        instanceId={props.instanceId}
        instancePath={props.instancePath}
        content={localProps.file}
      />
    ),
    data: () => props.data,
    refetch: () => props.refetch?.(),
    isLoading: () => !!props.isLoading,
    instanceId: () => props.instanceId,
  });

  const handleSearch = (query: string) =>
    table.getColumn('name')?.setFilterValue(query);

  createEffect(() => {
    if (props.searchQuery === undefined) {
      return;
    }

    handleSearch(props.searchQuery);
  });

  let tableElement: HTMLDivElement | undefined;

  const selectedRows = () => table.getSelectedRowModel();

  const clearSelection = () => {
    table.resetRowSelection(true);
    // The toolbar the focus was in is gone; hand it to the checkbox it replaced.
    tableElement?.querySelector<HTMLInputElement>('thead input')?.focus();
  };

  return (
    <>
      <span class='sr-only' role='status'>
        {selectedRows().rows.length > 0
          ? t('instance.selectedContents', {
              selected: selectedRows().rows.length,
              total: table.getFilteredRowModel().rows.length,
            })
          : ''}
      </span>
      <DataTable
        ref={(element) => (tableElement = element)}
        table={table}
        isLoading={props.isLoading}
        headerBar={{
          afterColumn: 'select',
          show: selectedRows().rows.length > 0,
          content: () => (
            <SelectionToolbar
              instanceId={props.instanceId}
              selectedRows={selectedRows()}
              totalCount={table.getFilteredRowModel().rows.length}
              onSelectAll={() => table.toggleAllRowsSelected(true)}
              onClear={clearSelection}
            />
          ),
        }}
      />
      <CombinedPagination
        class={cn('self-end', { hidden: table.getPageCount() < 2 })}
        siblingCount={1}
        count={table.getPageCount()}
        page={table.atoms.pagination.get().pageIndex + 1}
        onPageChange={(page) => table.setPageIndex(page - 1)}
      />
    </>
  );
};
