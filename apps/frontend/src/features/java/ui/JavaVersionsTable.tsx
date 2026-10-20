import type { ComponentProps } from 'solid-js';

import { Match, splitProps, Switch, type Component } from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import { DataTable, LoadError } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import { useJavaList } from '../api/javaQueries';
import { createJavaVersionsTable } from '../lib/createJavaVersionsTable';
import { useJavaVersionsTableData } from '../lib/useJavaVersionsTableData';

export type JavaVersionsTableProps = ComponentProps<'div'>;

export const JavaVersionsTable: Component<JavaVersionsTableProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [{ t }] = useTranslation();

  const javaVersions = useJavaList();
  const tableData = useJavaVersionsTableData(() => javaVersions.data);

  const table = createJavaVersionsTable({
    data: tableData,
  });

  return (
    <div class={cn('gap-2 flex flex-col', local.class)} {...others}>
      <Switch>
        <Match when={javaVersions.isError && !javaVersions.data}>
          <LoadError
            message={t('java.loadFailed')}
            retryLabel={t('common.retry')}
            onRetry={() => void javaVersions.refetch()}
          />
        </Match>
        <Match when={!javaVersions.isError || javaVersions.data}>
          <DataTable
            table={table}
            isLoading={javaVersions.isLoading && !javaVersions.isError}
          />
        </Match>
      </Switch>
    </div>
  );
};
