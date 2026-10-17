import { createTable } from '@tanstack/solid-table';
import { type Accessor } from 'solid-js';

import { appTableFeatures, useTranslation } from '@/shared/model';

import type { JavaVersion } from '../model/javaVersion';

import { createJavaVersionsColumns } from '../ui/JavaVersionsColumns';
import { useJavaVersionActions } from './useJavaVersionActions';

export interface JavaVersionsTableProps {
  data: Accessor<JavaVersion[]>;
}

export const createJavaVersionsTable = (props: JavaVersionsTableProps) => {
  const [{ t }] = useTranslation();

  const actions = useJavaVersionActions(() => props.data());

  const columns = createJavaVersionsColumns({
    t,
    onInstallRecommended: actions.installRecommended,
    isInstalling: actions.isInstalling,
    onDetect: actions.onDetect,
    onBrowse: actions.onBrowse,
  });

  const table = createTable({
    features: appTableFeatures,
    manualPagination: true,
    get data() {
      return props.data();
    },
    get columns() {
      return columns;
    },
    getRowId: (row) => `${row.majorVersion}`,
  });

  return table;
};
