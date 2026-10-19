import type { PolymorphicProps } from '@kobalte/core';
import type { DialogRootProps } from '@kobalte/core/dialog';

import { createMemo, Show, type ValidComponent } from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import {
  Button,
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/shared/ui-kit';

import {
  clearFinishedProgress,
  useProgressIndex,
} from '../model/progressStore';
import { getFinishedRoots } from '../model/progressTree';
import { ProgressDetails } from './ProgressDetails';

export type ProgressDetailsDialogProps = DialogRootProps;

const WIDTH_KEY = 'aether:progress-dialog-width';

export const ProgressDetailsDialog = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, ProgressDetailsDialogProps>,
) => {
  const [{ t }] = useTranslation();

  const index = useProgressIndex();

  const hasFinished = createMemo(() => getFinishedRoots(index()).length > 0);

  return (
    <Dialog {...props}>
      <DialogContent
        class='max-h-[calc(100%-128px)] flex flex-col overflow-hidden'
        resizable
        resizeStorageKey={WIDTH_KEY}
        defaultWidth={768}
        minWidth={384}
      >
        <DialogHeader class='mb-2'>
          <DialogTitle>{t('progress.dialogTitle')}</DialogTitle>
          <Show when={hasFinished()}>
            <Button
              class='self-start'
              variant='ghost'
              size='sm'
              onClick={clearFinishedProgress}
            >
              {t('events.clearFinished')}
            </Button>
          </Show>
        </DialogHeader>
        <ProgressDetails class='min-h-0 flex-1' index={index} />
      </DialogContent>
    </Dialog>
  );
};
