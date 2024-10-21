import { Component } from 'solid-js';

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
  const handleInstanceLaunch = async (instance: Instance) => {
    if (instance.installStage !== InstanceInstallStage.Installed) {
      return;
    }

    try {
      await launchMinecraftInstance(instance.nameId);
    } catch (e) {
      console.error(e);
    }
  };

  const handleInstanceRemove = async (instance: Instance) => {
    try {
      await removeMinecraftInstance(instance.nameId);
      refetchInstances();
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <InstanceContextMenu
      onPlay={() => handleInstanceLaunch(props.instance)}
      onRemove={() => handleInstanceRemove(props.instance)}
    >
      <ContextMenuTrigger>
        <InstanceCard {...props} />
      </ContextMenuTrigger>
    </InstanceContextMenu>
  );
};
