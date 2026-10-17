import { createTable } from '@tanstack/solid-table';
import { type Accessor } from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import { appTableFeatures } from '@/shared/ui-kit';

import { type StrictJavaVersion } from '../model/javaVersion';
import { createDetectedJavaVersionsColumns } from '../ui/DetectedJavaVersionsColumns';

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
