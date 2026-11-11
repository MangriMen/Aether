import type { Component } from 'solid-js';

import type { ContentFile } from '@/features/content';

import { getContentFileDisplayTitle } from '@/features/content';
import { useTranslation } from '@/shared/i18n';
import { CombinedDialog } from '@/shared/ui-kit';

export type RemoveContentsDialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  files: ContentFile[];
  onConfirm: () => void;
};

/** Asks before content files are deleted: the core removes them from the disk for good. */
export const RemoveContentsDialog: Component<RemoveContentsDialogProps> = (
  props,
) => {
  const [{ t }] = useTranslation();

  const title = () =>
    props.files.length === 1
      ? t('instance.removeContentTitle', {
          name: getContentFileDisplayTitle(props.files[0]!),
        })
      : t('instance.removeContentsTitle', { count: props.files.length });

  return (
    <CombinedDialog
      variant='destructive'
      open={props.open}
      onOpenChange={props.onOpenChange}
      header={title()}
      description={t('instance.removeContentsDescription')}
      buttonOkVariant='destructive'
      buttonOkText={t('common.remove')}
      onOk={props.onConfirm}
      onCancel={() => props.onOpenChange(false)}
    />
  );
};
