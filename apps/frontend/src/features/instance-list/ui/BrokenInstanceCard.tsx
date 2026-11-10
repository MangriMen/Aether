import type { Component } from 'solid-js';

import { createSignal, Show } from 'solid-js';

import {
  getBrokenReasonText,
  useRemoveBrokenInstance,
  useRevealInExplorer,
  type BrokenInstance,
} from '@/features/instance';
import { useTranslation } from '@/shared/i18n';
import { logError } from '@/shared/log';
import {
  Button,
  CombinedDialog,
  CombinedTooltip,
  Image,
} from '@/shared/ui-kit';

export type BrokenInstanceCardProps = {
  instance: BrokenInstance;
};

/**
 * An instance the launcher stores but cannot read: shown with why, so it does not just
 * disappear. It cannot be launched or edited; the user can look at its folder or delete it.
 */
export const BrokenInstanceCard: Component<BrokenInstanceCardProps> = (
  props,
) => {
  const [{ t }] = useTranslation();

  const [showRemoveModal, setShowRemoveModal] = createSignal(false);

  const { mutateAsync: removeBroken } = useRemoveBrokenInstance();
  const { mutateAsync: revealInExplorer } = useRevealInExplorer();

  const handleOpenFolder = () => {
    const dir = props.instance.dir;

    if (dir !== null) {
      revealInExplorer({ path: dir }).catch(logError);
    }
  };

  const handleRemove = () =>
    removeBroken(props.instance.id)
      .then(() => setShowRemoveModal(false))
      .catch(logError);

  return (
    <>
      <div class='group p-0.5 relative'>
        <div
          class='
            max-w-33 min-w-33 gap-2 rounded-md bg-card/card p-2 drop-shadow-md
            relative flex h-max flex-col overflow-hidden border
            border-destructive/50
          '
        >
          <Image
            class='size-28 opacity-50 grayscale'
            alt={t('instance.broken.title')}
          />
          <div class='flex flex-col'>
            <CombinedTooltip
              label={props.instance.id}
              as='div'
              class='w-full truncate'
            >
              {props.instance.id}
            </CombinedTooltip>
            <div class='text-sm text-destructive'>
              {t('instance.broken.title')}
            </div>
            <div class='text-xs text-muted-foreground wrap-break-word'>
              {getBrokenReasonText(props.instance.reason, t)}
            </div>
          </div>
          <div class='gap-1 flex flex-col'>
            <Show when={props.instance.dir !== null}>
              <Button size='sm' variant='outline' onClick={handleOpenFolder}>
                {t('instance.broken.openFolder')}
              </Button>
            </Show>
            <Button
              size='sm'
              variant='destructive'
              onClick={() => setShowRemoveModal(true)}
            >
              {t('common.remove')}
            </Button>
          </div>
        </div>
      </div>

      <CombinedDialog
        variant='destructive'
        open={showRemoveModal()}
        onOpenChange={setShowRemoveModal}
        header={t('instance.broken.removeTitle', { id: props.instance.id })}
        description={
          props.instance.dir === null
            ? t('instance.broken.removeRecord')
            : t('instance.broken.removeFolder')
        }
        buttonOkVariant='destructive'
        buttonOkText={t('common.remove')}
        onOk={handleRemove}
        onCancel={() => setShowRemoveModal(false)}
      />
    </>
  );
};
