import { CombinedDialog } from '@/shared/ui-kit';

import { closeDialog, showDialog } from './globalDialog';

// Every dialog gets its own id, so several can be open at once without replacing each other.
let dialogCounter = 0;

/**
 * Asks the user a yes-or-no question in a dialog. Resolves with the answer; closing the dialog
 * any other way is a "no".
 */
export const askConfirmation = (texts: {
  header?: string;
  description?: string;
  allow?: string;
  deny?: string;
}) =>
  new Promise<boolean>((resolve) => {
    const dialogId = `askConfirmation-${++dialogCounter}`;
    let answered = false;

    const answer = (allowed: boolean) => {
      if (answered) {
        return;
      }

      answered = true;
      closeDialog(dialogId);
      resolve(allowed);
    };

    showDialog(dialogId, CombinedDialog, {
      header: texts.header,
      description: texts.description,
      buttonOkText: texts.allow,
      buttonCancelText: texts.deny,
      onOk: () => answer(true),
      onCancel: () => answer(false),
      onOpenChange: (open: boolean) => {
        if (!open) {
          answer(false);
        }
      },
    });
  });
