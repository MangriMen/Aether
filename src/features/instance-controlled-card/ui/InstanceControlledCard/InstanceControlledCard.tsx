// eslint-disable-next-line import/named
import { UnlistenFn } from '@tauri-apps/api/event';
import {
  Component,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  Show,
} from 'solid-js';

import { isAetherLauncherError } from '@/shared/model';
import {
  Button,
  ContextMenuTrigger,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  showToast,
} from '@/shared/ui';

import {
  InstanceCard,
  InstanceContextMenu,
  listenProcess,
  openInstanceFolder,
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

  const id = createMemo(() => props.instance.id);

  const [instanceToRemove, setInstanceToRemove] = createSignal<
    Instance | undefined
  >();

  const stopProcessListener = () => processListenerUnlistenFn?.();

  const startProcessListener = (id: string) =>
    listenProcess((e) => {
      if (e.payload.instanceId === id) {
        setProcessPayload(e.payload);
        const isLaunched = e.payload.event === ProcessPayloadType.Launched;
        setIsLoading(false);
        setIsRunning(isLaunched);
      }
    });

  createEffect(() => {
    (async () => {
      processListenerUnlistenFn = await startProcessListener(id());
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
      await launchMinecraftInstance(instance.id);
    } catch (e) {
      setIsLoading(false);
      if (isAetherLauncherError(e)) {
        showToast({
          title: `Failed to launch ${instance.name}`,
          description: e.message,
          variant: 'destructive',
        });
      }
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
      if (isAetherLauncherError(e)) {
        showToast({
          title: `Failed to stop ${instance.name}`,
          description: e.message,
          variant: 'destructive',
        });
      }
    }
  };

  const handleOpenFolder = async (instance: Instance) => {
    if (
      instance.installStage !== InstanceInstallStage.Installed ||
      isLoading() ||
      isRunning()
    ) {
      return;
    }

    try {
      await openInstanceFolder(instance);
    } catch (e) {
      showToast({
        title: 'Failed to open folder',
        description: `Instance path: ${instance.path}`,
        variant: 'destructive',
      });
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
      await removeMinecraftInstance(instance.id);
      refetchInstances();
    } catch (e) {
      if (isAetherLauncherError(e)) {
        showToast({
          title: `Failed to remove ${instance.name}`,
          description: e.message,
          variant: 'destructive',
        });
      }
    }
  };

  return (
    <>
      <InstanceContextMenu
        isLoading={isLoading()}
        onPlay={() => handleInstanceLaunch(props.instance)}
        onOpenFolder={() => handleOpenFolder(props.instance)}
        onRemove={() => setInstanceToRemove(props.instance)}
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

      <Dialog
        open={!!instanceToRemove()}
        onOpenChange={() => setInstanceToRemove(undefined)}
      >
        <DialogContent>
          <DialogHeader class='text-lg'>
            Are you shure you want to delete instance {instanceToRemove()?.name}
            ?
          </DialogHeader>
          <DialogDescription class='text-base'>
            If you proceed, all data for your instance will be permanently
            erased, including your worlds. You will not be able to recover it.
          </DialogDescription>
          <DialogFooter>
            <Show when={instanceToRemove()}>
              {(instance) => (
                <Button
                  variant='destructive'
                  onClick={() => handleInstanceRemove(instance())}
                >
                  Remove
                </Button>
              )}
            </Show>
            <Button onClick={() => setInstanceToRemove(undefined)}>
              Cancel
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
};
