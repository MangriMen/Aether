import type { DialogRootProps } from '@kobalte/core/dialog';

import { createMemo, splitProps, type Component } from 'solid-js';

import type { Java } from '@/entities/java';

import { useTranslation } from '@/shared/model';
import {
  DataTable,
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/shared/ui';

import type { JavaVersion } from '../../model';

import { createDetectedJavaVersionsTable } from '../../lib/createDetectedJavaVersionsTable';

export type DetectJavaDialogProps = DialogRootProps & {
  versions?: Java[];
  majorVersion?: string;
  selectedVersion?: JavaVersion;
  onSelect?: (version: JavaVersion) => void;
};

export const DetectJavaDialog: Component<DetectJavaDialogProps> = (props) => {
  const [local, others] = splitProps(props, [
    'versions',
    'majorVersion',
    'selectedVersion',
    'onSelect',
  ]);

  const [{ t }] = useTranslation();

  const mappedVersions = createMemo(
    () =>
      local.versions?.map<JavaVersion>((v) => ({
        majorVersion: v.majorVersion.toString(),
        version: v.version,
        path: v.path,
      })) ?? [],
  );

  const majorVersion = createMemo(
    () => local.majorVersion ?? local.versions?.[0]?.majorVersion ?? '?',
  );

  const table = createDetectedJavaVersionsTable({
    data: () => mappedVersions(),
    currentVersion: () => local.selectedVersion,
    onSelect: (version) => local.onSelect?.(version),
  });

  return (
    <Dialog {...others}>
      <DialogContent class='max-w-3xl gap-4'>
        <DialogHeader>
          <DialogTitle>
            {t('detectJava.dialogTitle', {
              majorVersion: majorVersion(),
            })}
          </DialogTitle>
        </DialogHeader>

        <DataTable
          noContentPlaceholder={t('detectJava.installationsNotFound')}
          table={table}
        />
      </DialogContent>
    </Dialog>
  );
};
