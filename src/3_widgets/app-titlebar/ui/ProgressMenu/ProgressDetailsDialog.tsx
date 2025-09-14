import type { PolymorphicProps } from '@kobalte/core';
import type { DialogRootProps } from '@kobalte/core/dialog';

import { createMemo, type ValidComponent } from 'solid-js';

import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/shared/ui';

import { useProgressStore } from '../../model';
import { ProgressDetails } from './ProgressDetails';

export type ProgressDetailsDialogProps = DialogRootProps;

export const ProgressDetailsDialog = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, ProgressDetailsDialogProps>,
) => {
  const [progressStore] = useProgressStore();

  const progressValues = createMemo(() =>
    Object.values(progressStore.payloads),
  );

  return (
    <Dialog {...props}>
      <DialogContent class='max-h-[calc(100%-128px)] max-w-[calc(100%-128px)] bg-secondary-dark'>
        <DialogHeader>
          <DialogTitle>Progress details</DialogTitle>
        </DialogHeader>
        <ProgressDetails payloads={progressValues} />
      </DialogContent>
    </Dialog>
  );
};
