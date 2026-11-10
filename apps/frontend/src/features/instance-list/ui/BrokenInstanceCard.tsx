import type { Component } from 'solid-js';

import { createSignal } from 'solid-js';

import {
  BrokenInstanceTile,
  useRemoveBrokenInstance,
  useRevealInExplorer,
  type BrokenInstance,
} from '@/features/instance';
import { useTranslation } from '@/shared/i18n';
import { logError } from '@/shared/log';
import { CombinedDialog, ContextMenuTrigger } from '@/shared/ui-kit';

import { BrokenInstanceContextMenu } from './BrokenInstanceContextMenu';
import { BrokenInstanceDetailsDialog } from './BrokenInstanceDetailsDialog';

export type BrokenInstanceCardProps = {
  instance: BrokenInstance;
};

/**
 * An instance the launcher stores but cannot read, in the list with the others so it does not
 * just disappear. It cannot be launched or edited: the click shows what is known of it, and the
 * user can look at its folder or delete it, here or from the menu.
 */
export const BrokenInstanceCard: Component<BrokenInstanceCardProps> = (
  props,
) => {
  const [{ t }] = useTranslation();

  const [showDetails, setShowDetails] = createSignal(false);
  const [showRemoveModal, setShowRemoveModal] = createSignal(false);

  let wrapper: HTMLDivElement | undefined;

  const { mutateAsync: removeBroken } = useRemoveBrokenInstance();
  const { mutateAsync: revealInExplorer } = useRevealInExplorer();

  const handleOpenFolder = () => {
    const dir = props.instance.dir;

    if (dir !== null) {
      revealInExplorer({ path: dir }).catch(logError);
    }
  };

  const askToRemove = () => {
    setShowDetails(false);
    setShowRemoveModal(true);
  };

  const handleRemove = () =>
    removeBroken(props.instance.id)
      .then(() => setShowRemoveModal(false))
      .catch(logError);

  return (
    <>
      <div class='group p-0.5 relative' ref={(element) => (wrapper = element)}>
        <BrokenInstanceContextMenu
          onOpenFolder={handleOpenFolder}
          onRemove={askToRemove}
          disableOpenFolder={props.instance.dir === null}
        >
          <ContextMenuTrigger
            as={BrokenInstanceTile}
            data-context-menu-safe='true'
            aria-haspopup='dialog'
            onClick={() => setShowDetails(true)}
            instance={props.instance}
          />
        </BrokenInstanceContextMenu>
      </div>

      <BrokenInstanceDetailsDialog
        instance={props.instance}
        open={showDetails()}
        onOpenChange={setShowDetails}
        onOpenFolder={handleOpenFolder}
        onRemove={askToRemove}
        // Without a trigger the focus would fall to the page, and the arrow keys would stop.
        onCloseAutoFocus={(event) => {
          event.preventDefault();
          wrapper?.querySelector('button')?.focus();
        }}
      />

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
