import type { Component } from 'solid-js';

import { useNavigate } from '@solidjs/router';
import { createMemo, createSignal } from 'solid-js';

import type { InstanceCardProps } from '@/entities/instances';

import {
  InstanceCard,
  InstanceContextMenu,
  useInstanceActions,
  useInstanceDir,
  useRevealInExplorer,
  useRunningInstancesContext,
} from '@/entities/instances';
import { preventAll } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedDialog, ContextMenuTrigger } from '@/shared/ui';

export type InstanceControlledCardProps = Pick<
  InstanceCardProps,
  'instance' | 'instanceActionButton'
>;

export const InstanceControlledCard: Component<InstanceControlledCardProps> = (
  props,
) => {
  const navigate = useNavigate();

  const id = createMemo(() => props.instance.id);

  const instancePath = useInstanceDir(() => id());

  const [{ t }] = useTranslation();

  const [showRemoveModal, setShowRemoveModal] = createSignal(false);

  const openRemoveModal = () => setShowRemoveModal(true);
  const closeRemoveModal = () => {
    setShowRemoveModal(false);
  };

  const { launch: launchInstance, remove: removeInstance } =
    useInstanceActions();

  const handleLaunch = (e: MouseEvent) => {
    preventAll(e);
    launchInstance(props.instance);
  };

  const handleRemove = () =>
    removeInstance(props.instance).then(() => closeRemoveModal());

  const { mutateAsync: revealInExplorer } = useRevealInExplorer();
  const handleOpenFolder = () => {
    if (!instancePath.data) {
      return;
    }

    revealInExplorer({ path: instancePath.data });
  };

  const handleOpenSettings = () => {
    navigate(`/instance-settings/${encodeURIComponent(props.instance.id)}`);
  };

  const goToInstancePage = () => {
    navigate(`/instances/${encodeURIComponent(props.instance.id)}`);
  };

  const context = useRunningInstancesContext();
  const runningInstanceData = createMemo(() => context[0].instances[id()]);

  return (
    <>
      <InstanceContextMenu
        isLoading={runningInstanceData()?.isLoading}
        onPlay={handleLaunch}
        onOpenFolder={handleOpenFolder}
        onOpenSettings={handleOpenSettings}
        onRemove={openRemoveModal}
        disableOpenFolder={!instancePath}
      >
        <ContextMenuTrigger
          as={InstanceCard}
          onClick={goToInstancePage}
          isLoading={runningInstanceData()?.isLoading}
          isRunning={runningInstanceData()?.isRunning}
          {...props}
        />
      </InstanceContextMenu>

      <CombinedDialog
        variant='destructive'
        open={showRemoveModal()}
        onOpenChange={setShowRemoveModal}
        header={t('instance.removeTitle', { name: props.instance.name })}
        description={t('instance.removeDescription')}
        buttonOkText={t('common.remove')}
        onOk={handleRemove}
        onCancel={closeRemoveModal}
      />
    </>
  );
};
