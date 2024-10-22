import { Component, createSignal, onCleanup } from 'solid-js';

import { ContextMenuTrigger } from '@/shared/ui';

import {
  InstanceCard,
  InstanceContextMenu,
  refetchInstances,
} from '@/entities/instance';
import {
  Instance,
  InstanceInstallStage,
  launchMinecraftInstance,
  removeMinecraftInstance,
} from '@/entities/minecraft';

import { InstanceControlledCardProps } from './types';

export const InstanceControlledCard: Component<InstanceControlledCardProps> = (
  props,
) => {
  const [isLoading, setIsLoading] = createSignal(false);
  const [launchTimer, setLaunchTimer] = createSignal<number | undefined>(
    undefined,
  );

  const handleInstanceLaunch = async (instance: Instance) => {
    if (
      instance.installStage !== InstanceInstallStage.Installed ||
      isLoading()
    ) {
      return;
    }

    try {
      setIsLoading(true);
      await launchMinecraftInstance(instance.nameId);
      // TODO: remove when launch event is implemented
      setLaunchTimer(
        setTimeout(() => {
          setIsLoading(false);
        }, 10000),
      );
    } catch (e) {
      console.error(e);
    }
  };

  const handleInstanceRemove = async (instance: Instance) => {
    if (
      instance.installStage !== InstanceInstallStage.Installed ||
      isLoading()
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

  onCleanup(() => {
    clearTimeout(launchTimer());
    setLaunchTimer(undefined);
  });

  return (
    <InstanceContextMenu
      isLoading={isLoading()}
      onPlay={() => handleInstanceLaunch(props.instance)}
      onRemove={() => handleInstanceRemove(props.instance)}
    >
      <ContextMenuTrigger>
        <InstanceCard
          isLoading={isLoading()}
          onLaunchClick={() => handleInstanceLaunch(props.instance)}
          {...props}
        />
      </ContextMenuTrigger>
    </InstanceContextMenu>
  );
};
