import { createSolidTable, getCoreRowModel } from '@tanstack/solid-table';
import { type Accessor } from 'solid-js';

import { useTranslation } from '@/shared/model';

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

  const table = createSolidTable({
    get data() {
      return props.data();
    },
    get columns() {
      return columns();
    },
    getCoreRowModel: getCoreRowModel(),
    getRowId: (row) => `${row.majorVersion}`,
  });

  return table;
};
