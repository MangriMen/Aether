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
import { ROUTES } from '@/shared/config';
import { preventAll } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import {
  CombinedDialog,
  CombinedTooltip,
  ContextMenuTrigger,
} from '@/shared/ui';

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
    navigate(ROUTES.INSTANCE_DIALOG(props.instance.id));
  };

  const goToInstancePage = () => {
    navigate(ROUTES.INSTANCE(props.instance.id));
  };

  const context = useRunningInstancesContext();
  const runningInstanceData = createMemo(() => context[0].instances[id()]);

  return (
    <>
      <div class='group relative p-0.5'>
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
            overrideIcon={props.instance.iconPath ?? undefined}
            {...props}
          />
        </InstanceContextMenu>

        <props.instanceActionButton
          class='absolute right-4 top-[72px] z-50 p-0 opacity-0 transition-[bottom,opacity] focus-within:bottom-1/4 focus-within:opacity-100 disabled:opacity-0 group-hover:bottom-1/4 group-hover:opacity-100'
          instance={props.instance}
        />
      </div>

      <CombinedDialog
        variant='destructive'
        open={showRemoveModal()}
        onOpenChange={setShowRemoveModal}
        header={
          <CombinedTooltip
            label={props.instance.name}
            as='span'
            class='line-clamp-2'
          >
            {t('instance.removeTitle', { name: props.instance.name })}
          </CombinedTooltip>
        }
        description={t('instance.removeDescription')}
        buttonOkVariant='destructive'
        buttonOkText={t('common.remove')}
        onOk={handleRemove}
        onCancel={closeRemoveModal}
      />
    </>
  );
};
