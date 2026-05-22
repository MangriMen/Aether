import type { DialogRootProps } from '@kobalte/core/dialog';

import { createMemo, splitProps, type Component } from 'solid-js';

import type { Java } from '@/entities/java';

import { useTranslation } from '@/shared/model';
import { DataTable, Dialog, DialogContent, DialogHeader } from '@/shared/ui';

import type { JavaVersion } from '../../model';

import { createDetectedJavaVersionsTable } from '../../lib/createDetectedJavaVersionsTable';

export type DetectJavaDialogProps = DialogRootProps & {
  versions?: Java[];
  selectedVersion?: JavaVersion;
  onSelect?: (version: JavaVersion) => void;
};

export const DetectJavaDialog: Component<DetectJavaDialogProps> = (props) => {
  const [local, others] = splitProps(props, [
    'versions',
    'selectedVersion',
    'onSelect',
  ]);

  const [{ t }] = useTranslation();

  const mappedVersions = createMemo(
    () =>
      local.versions?.map<JavaVersion>((v) => ({
        majorVersion: v.majorVersion.toString(),
        path: v.path,
      })) ?? [],
  );

  const table = createDetectedJavaVersionsTable({
    data: () => mappedVersions(),
    currentVersion: () => local.selectedVersion,
    onSelect: (version) => local.onSelect?.(version),
  });

  return (
    <Dialog {...others}>
      <DialogContent class='max-w-3xl'>
        <DialogHeader>{t('detectJava.dialogTitle')}</DialogHeader>

        <DataTable
          noContentPlaceholder={t('detectJava.installationsNotFound')}
          table={table}
        />
      </DialogContent>
    </Dialog>
  );
};
