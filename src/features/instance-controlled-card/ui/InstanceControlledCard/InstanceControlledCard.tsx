// eslint-disable-next-line import/named
import { UnlistenFn } from '@tauri-apps/api/event';
import {
  Component,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
} from 'solid-js';

import { ContextMenuTrigger } from '@/shared/ui';

import {
  InstanceCard,
  InstanceContextMenu,
  listenProcess,
  refetchInstances,
} from '@/entities/instance';
import {
  Instance,
  InstanceInstallStage,
  launchMinecraftInstance,
  ProcessPayload,
  ProcessPayloadType,
  removeMinecraftInstance,
  stopMinecraftInstance,
} from '@/entities/minecraft';

import { InstanceControlledCardProps } from './types';

export const InstanceControlledCard: Component<InstanceControlledCardProps> = (
  props,
) => {
  const [isRunning, setIsRunning] = createSignal(false);
  const [isLoading, setIsLoading] = createSignal(false);

  const [processPayload, setProcessPayload] = createSignal<
    ProcessPayload | undefined
  >(undefined);

  let processListenerUnlistenFn: UnlistenFn | undefined;

  const nameId = createMemo(() => props.instance.nameId);

  const stopProcessListener = () => processListenerUnlistenFn?.();

  const startProcessListener = (nameId: string) =>
    listenProcess((e) => {
      if (e.payload.instanceNameId === nameId) {
        setProcessPayload(e.payload);
        const isLaunched = e.payload.event === ProcessPayloadType.Launched;
        setIsLoading(false);
        setIsRunning(isLaunched);
      }
    });

  createEffect(() => {
    (async () => {
      processListenerUnlistenFn = await startProcessListener(nameId());
    })();
  });

  onCleanup(() => {
    stopProcessListener();
  });

  const handleInstanceLaunch = async (instance: Instance) => {
    if (
      instance.installStage !== InstanceInstallStage.Installed ||
      isLoading() ||
      isRunning()
    ) {
      return;
    }

    try {
      setIsLoading(true);
      await launchMinecraftInstance(instance.nameId);
    } catch (e) {
      setIsLoading(false);
      console.error(e);
    }
  };

  const handleInstanceStop = async (instance: Instance) => {
    if (
      instance.installStage !== InstanceInstallStage.Installed ||
      isLoading()
    ) {
      return;
    }

    const uuid = processPayload()?.uuid;
    if (uuid === undefined) {
      return;
    }

    try {
      setIsLoading(true);
      await stopMinecraftInstance(uuid);
    } catch (e) {
      setIsLoading(false);
      console.error(e);
    }
  };

  const handleInstanceRemove = async (instance: Instance) => {
    if (
      instance.installStage !== InstanceInstallStage.Installed ||
      isLoading() ||
      isRunning()
    ) {
      return;
    }

    try {
      await removeMinecraftInstance(instance.nameId);
      refetchInstances();
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <InstanceContextMenu
      isLoading={isLoading()}
      onPlay={() => handleInstanceLaunch(props.instance)}
      onRemove={() => handleInstanceRemove(props.instance)}
    >
      <ContextMenuTrigger>
        <InstanceCard
          isRunning={isRunning()}
          isLoading={isLoading()}
          onLaunchClick={() => handleInstanceLaunch(props.instance)}
          onStopClick={() => handleInstanceStop(props.instance)}
          {...props}
        />
      </ContextMenuTrigger>
    </InstanceContextMenu>
  );
};
