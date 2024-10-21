import {
  Component,
  createEffect,
  For,
  Match,
  Show,
  splitProps,
  Switch,
} from 'solid-js';

import { cn } from '@/shared/lib';
import { ContextMenuTrigger } from '@/shared/ui';

import {
  getInstances,
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

import { InstancesPanelProps } from './types';

export const InstancesPanel: Component<InstancesPanelProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const instances = getInstances();

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

  createEffect(() => {
    refetchInstances();
  });

  return (
    <div class={cn('flex flex-wrap gap-4', local.class)} {...others}>
      <Switch>
        <Match when={!instances.loading}>
          <Show
            when={instances()?.length}
            fallback={<span class='m-auto'>You don't have instances</span>}
          >
            <For each={instances()}>
              {(instance) => (
                <InstanceContextMenu
                  onPlay={() => handleInstanceLaunch(instance)}
                  onRemove={() => handleInstanceRemove(instance)}
                >
                  <ContextMenuTrigger>
                    <InstanceCard
                      instance={instance}
                      onLaunchClick={() => handleInstanceLaunch(instance)}
                    />
                  </ContextMenuTrigger>
                </InstanceContextMenu>
              )}
            </For>
          </Show>
        </Match>
      </Switch>
    </div>
  );
};
