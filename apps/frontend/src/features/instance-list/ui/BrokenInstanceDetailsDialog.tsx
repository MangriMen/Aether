import type { Component } from 'solid-js';

import { Show } from 'solid-js';

import { getBrokenReasonText, type BrokenInstance } from '@/features/instance';
import { useTranslation } from '@/shared/i18n';
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/shared/ui-kit';

export type BrokenInstanceDetailsDialogProps = {
  instance: BrokenInstance;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onOpenFolder: () => void;
  onRemove: () => void;
  /** Where the focus goes when the dialog closes: back to the card it came from. */
  onCloseAutoFocus?: (event: Event) => void;
};

/** What is known of a damaged instance, in full and to be copied, and what can be done. */
export const BrokenInstanceDetailsDialog: Component<
  BrokenInstanceDetailsDialogProps
> = (props) => {
  const [{ t }] = useTranslation();

  return (
    <Dialog open={props.open} onOpenChange={props.onOpenChange}>
      <DialogContent class='max-w-md' onCloseAutoFocus={props.onCloseAutoFocus}>
        <DialogHeader>
          <DialogTitle>{t('instance.broken.title')}</DialogTitle>
          <DialogDescription>
            {t('instance.broken.description')}
          </DialogDescription>
        </DialogHeader>
        <dl class='gap-x-3 gap-y-2 text-sm grid grid-cols-[auto_1fr]'>
          <dt class='text-muted-foreground'>{t('instance.broken.id')}</dt>
          <dd class='break-all select-text'>{props.instance.id}</dd>
          <dt class='text-muted-foreground'>{t('instance.broken.reason')}</dt>
          <dd class='wrap-break-word whitespace-pre-wrap select-text'>
            {getBrokenReasonText(props.instance.reason, t)}
          </dd>
          <dt class='text-muted-foreground'>{t('instance.broken.folder')}</dt>
          <dd class='font-mono text-xs break-all select-text'>
            {props.instance.dir ?? t('instance.broken.noFolder')}
          </dd>
        </dl>
        <DialogFooter>
          <Button variant='destructive' onClick={props.onRemove}>
            {t('common.remove')}
          </Button>
          <Show when={props.instance.dir !== null}>
            <Button variant='outline' onClick={props.onOpenFolder}>
              {t('instance.openFolder')}
            </Button>
          </Show>
          <Button variant='secondary' onClick={() => props.onOpenChange(false)}>
            {t('common.close')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
