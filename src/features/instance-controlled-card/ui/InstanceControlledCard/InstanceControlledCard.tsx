import { useNavigate } from '@solidjs/router';
import { Component, createMemo, createSignal } from 'solid-js';

import { preventAll } from '@/shared/lib';
import { CombinedDialog, ContextMenuTrigger } from '@/shared/ui';

import {
  InstanceCard,
  InstanceContextMenu,
  useInstanceActions,
  useRunningInstancesContext,
} from '@/entities/instance';

import { InstanceControlledCardProps } from './types';

export const InstanceControlledCard: Component<InstanceControlledCardProps> = (
  props,
) => {
  const navigate = useNavigate();

  const id = createMemo(() => props.instance.id);

  const [showRemoveModal, setShowRemoveModal] = createSignal(false);

  const openRemoveModal = () => setShowRemoveModal(true);
  const closeRemoveModal = () => {
    setShowRemoveModal(false);
  };

  const { launchInstance, removeInstance, stopInstance, openFolder } =
    useInstanceActions();

  const handleLaunch = (e: MouseEvent) => {
    preventAll(e);
    launchInstance(props.instance);
  };
  const handleStop = (e: MouseEvent) => {
    preventAll(e);
    stopInstance(props.instance);
  };

  const handleRemove = () => removeInstance(props.instance);

  const handleOpenFolder = () => openFolder(props.instance);

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
        onRemove={openRemoveModal}
      >
        <ContextMenuTrigger
          as={InstanceCard}
          isRunning={runningInstanceData()?.isRunning}
          isLoading={runningInstanceData()?.isLoading}
          onClick={goToInstancePage}
          onLaunchClick={handleLaunch}
          onStopClick={handleStop}
          {...props}
        />
      </InstanceContextMenu>

      <CombinedDialog
        variant='destructive'
        open={showRemoveModal()}
        onOpenChange={setShowRemoveModal}
        header={`Are you shure you want to delete instance ${props.instance?.name}?`}
        description='If you proceed, all data for your instance will be permanently
            erased, including your worlds. You will not be able to recover it.'
        buttonOkText='Remove'
        onOk={handleRemove}
        onCancel={closeRemoveModal}
      />
    </>
  );
};
