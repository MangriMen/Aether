import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/shared/ui';
import { createMemo, type ValidComponent } from 'solid-js';
import { ProgressDetails } from './ProgressDetails';
import type { DialogRootProps } from '@kobalte/core/dialog';
import type { PolymorphicProps } from '@kobalte/core';
import { useProgressStore } from '../../model';

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
