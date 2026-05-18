import { createSolidTable, getCoreRowModel } from '@tanstack/solid-table';
import { type Accessor } from 'solid-js';

import { useTranslation } from '@/shared/model';

import type { JavaVersion } from '../model';

import { createJavaVersionColumns } from '../model';
import { useJavaVersionActions } from './useJavaVersionActions';

export interface JavaVersionsTableProps {
  data: Accessor<JavaVersion[]>;
}

export const createJavaVersionsTable = (props: JavaVersionsTableProps) => {
  const [{ t }] = useTranslation();

  const actions = useJavaVersionActions();

  const columns = createJavaVersionColumns({
    t,
    onInstallRecommended: actions.installRecommended,
    isInstalling: actions.isInstalling,
  });

  const table = createSolidTable({
    get data() {
      return props.data();
    },
    get columns() {
      return columns();
    },
    getCoreRowModel: getCoreRowModel(),
  });

  return { table, columns };
};
