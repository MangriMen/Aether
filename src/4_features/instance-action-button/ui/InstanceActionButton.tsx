import type { Component } from 'solid-js';

import { createMemo, Show, splitProps } from 'solid-js';

import type { Instance } from '@/entities/instances';
import type { IconButtonProps } from '@/shared/ui';

import {
  InstanceInstallStage,
  InstancePlayButton,
  InstanceStopButton,
  useInstanceActions,
  useRunningInstancesContext,
} from '@/entities/instances';

export type InstanceActionButtonProps = {
  instance: Instance;
} & IconButtonProps;

export const InstanceActionButton: Component<InstanceActionButtonProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['instance']);

  const [context, { get }] = useRunningInstancesContext();
  const runningInstance = createMemo(() => get(context, local.instance.id));

  const { launch: launchInstance, stop: stopInstance } = useInstanceActions();

  const isInstanceInstalled = createMemo(
    () => local.instance.installStage === InstanceInstallStage.Installed,
  );

  const isInstanceInstalling = createMemo(
    () => local.instance.installStage === InstanceInstallStage.Installing,
  );

  const isPlayButtonLoading = createMemo(
    () => isInstanceInstalling() || runningInstance()?.isLoading,
  );

  const isPlayButton = createMemo(() => !runningInstance()?.isRunning);

  const isPlayButtonDisabled = createMemo(
    () =>
      !isInstanceInstalled() ||
      runningInstance()?.isRunning ||
      runningInstance()?.isLoading,
  );

  const handleLaunch = (e: MouseEvent) => {
    e.stopPropagation();
    launchInstance(local.instance);
  };
  const handleStop = (e: MouseEvent) => {
    e.stopPropagation();
    stopInstance(local.instance);
  };

  return (
    <Show
      fallback={<InstanceStopButton onClick={handleStop} {...others} />}
      when={isPlayButton()}
    >
      <InstancePlayButton
        disabled={isPlayButtonDisabled()}
        loading={isPlayButtonLoading()}
        onClick={handleLaunch}
        {...others}
      />
    </Show>
  );
};
