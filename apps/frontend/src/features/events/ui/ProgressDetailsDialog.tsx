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
  useProgressStore,
} from '../model/progressStore';
import { getFinishedRoots } from '../model/progressTree';
import { ProgressDetails } from './ProgressDetails';

export type ProgressDetailsDialogProps = DialogRootProps;

export const ProgressDetailsDialog = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, ProgressDetailsDialogProps>,
) => {
  const [{ t }] = useTranslation();

  const [progressStore] = useProgressStore();

  const progressValues = createMemo(() =>
    Object.values(progressStore.payloads),
  );

  const hasFinished = createMemo(
    () => getFinishedRoots(progressValues()).length > 0,
  );

  return (
    <Dialog {...props}>
      <DialogContent
        class='
          2xl:max-w-(--breakpoint-2xl)
          max-h-[calc(100%-128px)] w-full max-w-[calc(100%-128px)]
        '
      >
        <DialogHeader>
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
        <ProgressDetails payloads={progressValues} />
      </DialogContent>
    </Dialog>
  );
};
