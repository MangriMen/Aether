import { createSolidTable, getCoreRowModel } from '@tanstack/solid-table';
import { open } from '@tauri-apps/plugin-dialog';
import { type Accessor } from 'solid-js';

import { useDiscoverJava, useEditJava } from '@/entities/java';
import { closeDialog, showDialog, useTranslation } from '@/shared/model';

import type { JavaVersion } from '../model';

import { createJavaVersionColumns } from '../model';
import { DetectJavaDialog } from '../ui/JavaPane/DetectJavaDialog';
import { parseJavaVersion } from './parseJavaVersion';
import { useJavaVersionActions } from './useJavaVersionActions';

export interface JavaVersionsTableProps {
  data: Accessor<JavaVersion[]>;
}

export const createJavaVersionsTable = (props: JavaVersionsTableProps) => {
  const [{ t }] = useTranslation();

  const actions = useJavaVersionActions();

  const editJava = useEditJava();

  const discoverJava = useDiscoverJava();

  const handleDetect = async (version: string) => {
    const result = await discoverJava.mutateAsync();

    const versionNum = parseJavaVersion(version);
    const filtered = result.filter((java) => java.majorVersion === versionNum);

    const dialogId = 'detect-java';

    const handleSelect = async (newVersion: JavaVersion) => {
      if (newVersion.majorVersion !== version) {
        return;
      }

      if (!versionNum || !newVersion.path) {
        return;
      }

      await editJava.mutateAsync({
        majorVersion: versionNum,
        path: newVersion.path,
      });

      closeDialog(dialogId);
    };

    const selectedVersion = props
      .data()
      .find((java) => java.majorVersion === version);

    showDialog(dialogId, DetectJavaDialog, {
      versions: filtered,
      selectedVersion,
      onSelect: handleSelect,
      majorVersion: version,
    });
  };

  const handleBrowse = async (version: string) => {
    const versionNum = parseJavaVersion(version);

    if (!versionNum) {
      return;
    }

    const path = await open({
      title: 'Select javaw executable',
      filters: [
        { name: 'Java Window Executable', extensions: ['exe'] },
        { name: 'All', extensions: ['*'] },
      ],
      multiple: false,
    });

    if (!path) {
      return;
    }

    editJava.mutateAsync({
      majorVersion: versionNum,
      path,
    });
  };

  const columns = createJavaVersionColumns({
    t,
    onInstallRecommended: actions.installRecommended,
    isInstalling: actions.isInstalling,
    onDetect: handleDetect,
    onBrowse: handleBrowse,
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
