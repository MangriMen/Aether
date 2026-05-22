import { createSolidTable, getCoreRowModel } from '@tanstack/solid-table';
import { type Accessor } from 'solid-js';

import { useTranslation } from '@/shared/model';

import type { JavaVersion } from '../model';

import { createDetectedJavaVersionColumns } from '../model';

export interface DetectedJavaVersionsTableProps {
  data: Accessor<JavaVersion[]>;
  currentVersion?: Accessor<JavaVersion | undefined>;
  onSelect?: (version: JavaVersion) => void;
}

export const createDetectedJavaVersionsTable = (
  props: DetectedJavaVersionsTableProps,
) => {
  const [{ t }] = useTranslation();

  const columns = createDetectedJavaVersionColumns({
    t,
    onSelect: (version) => props.onSelect?.(version),
    // eslint-disable-next-line solid/reactivity
    currentVersion: props.currentVersion,
  });

  const table = createSolidTable({
    get data() {
      return props.data();
    },
    get columns() {
      return columns();
    },
    getRowId: (row) => row.majorVersion,
    getCoreRowModel: getCoreRowModel(),
  });

  return table;
};
