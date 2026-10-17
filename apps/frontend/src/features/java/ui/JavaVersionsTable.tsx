import type { ComponentProps } from 'solid-js';

import { splitProps, type Component } from 'solid-js';

import { cn } from '@/shared/lib';
import { DataTable } from '@/shared/ui';

import { useJavaList } from '../api/javaQueries';
import { createJavaVersionsTable } from '../lib/createJavaVersionsTable';
import { useJavaVersionsTableData } from '../lib/useJavaVersionsTableData';

export type JavaVersionsTableProps = ComponentProps<'div'>;

export const JavaVersionsTable: Component<JavaVersionsTableProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const javaVersions = useJavaList();
  const tableData = useJavaVersionsTableData(() => javaVersions.data);

  const table = createJavaVersionsTable({
    data: tableData,
  });

  return (
    <div class={cn('gap-2 flex flex-col', local.class)} {...others}>
      <DataTable table={table} isLoading={javaVersions.isLoading} />
    </div>
  );
};
