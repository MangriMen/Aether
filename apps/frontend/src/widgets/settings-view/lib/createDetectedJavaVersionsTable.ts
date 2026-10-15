import { createTable } from '@tanstack/solid-table';
import { type Accessor } from 'solid-js';

import { appTableFeatures, useTranslation } from '@/shared/model';

import {
  createDetectedJavaVersionsColumns,
  type StrictJavaVersion,
} from '../model';

export interface DetectedJavaVersionsTableProps {
  data: Accessor<StrictJavaVersion[]>;
  selectedVersionPath?: Accessor<string | undefined>;
  onSelect?: (version: StrictJavaVersion) => void;
}

export const createDetectedJavaVersionsTable = (
  props: DetectedJavaVersionsTableProps,
) => {
  const [{ t }] = useTranslation();

  const columns = createDetectedJavaVersionsColumns({
    t,
    onSelect: (version) => props.onSelect?.(version),
    selectedVersionPath: () => props.selectedVersionPath?.(),
  });

  const table = createTable({
    features: appTableFeatures,
    manualPagination: true,
    get data() {
      return props.data();
    },
    get columns() {
      return columns();
    },
    getRowId: (row) => `${row.majorVersion}`,
  });

  return table;
};
