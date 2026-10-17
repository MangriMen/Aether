import type { DialogRootProps } from '@kobalte/core/dialog';
import type { Accessor } from 'solid-js';

import { createMemo, splitProps, type Component } from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import {
  DataTable,
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/shared/ui-kit';

import type { Java } from '../model/java';
import type { StrictJavaVersion } from '../model/javaVersion';

import { createDetectedJavaVersionsTable } from '../lib/createDetectedJavaVersionsTable';

export type DetectJavaDialogProps = DialogRootProps & {
  versions?: Java[];
  majorVersion?: number;
  selectedVersionPath?: Accessor<string | undefined>;
  onSelect?: (version: StrictJavaVersion) => void;
};

export const DetectJavaDialog: Component<DetectJavaDialogProps> = (props) => {
  const [local, others] = splitProps(props, [
    'versions',
    'majorVersion',
    'selectedVersionPath',
    'onSelect',
  ]);

  const [{ t }] = useTranslation();

  const mappedVersions = createMemo(
    () =>
      local.versions?.map<StrictJavaVersion>((v) => ({
        majorVersion: v.majorVersion,
        version: v.version,
        path: v.path,
      })) ?? [],
  );

  const majorVersion = createMemo(
    () => local.majorVersion ?? local.versions?.[0]?.majorVersion ?? '?',
  );

  const table = createDetectedJavaVersionsTable({
    data: () => mappedVersions(),
    selectedVersionPath: () => local.selectedVersionPath?.(),
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
