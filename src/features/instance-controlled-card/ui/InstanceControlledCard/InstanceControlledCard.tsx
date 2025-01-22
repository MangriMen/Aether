import { useNavigate } from '@solidjs/router';
// eslint-disable-next-line import/named
import { Component, createMemo, createSignal, Show } from 'solid-js';

import { preventAll } from '@/shared/lib';
import {
  Button,
  ContextMenuTrigger,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
} from '@/shared/ui';

import {
  InstanceCard,
  InstanceContextMenu,
  useInstanceActions,
  useRunningInstancesContext,
  Instance,
} from '@/entities/instance';

import { InstanceControlledCardProps } from './types';

export const InstanceControlledCard: Component<InstanceControlledCardProps> = (
  props,
) => {
  const navigate = useNavigate();

  const id = createMemo(() => props.instance.id);

  const context = useRunningInstancesContext();
  const runningInstance = createMemo(() => context[0].instances[id()]);

  const {
    handleInstanceLaunch,
    handleInstanceRemove,
    handleInstanceStop,
    handleOpenFolder,
  } = useInstanceActions();

  const [instanceToRemove, setInstanceToRemove] = createSignal<
    Instance | undefined
  >();

  const goToInstancePage = () => {
    navigate(`/instances/${props.instance.id}`);
  };

  return (
    <>
      <InstanceContextMenu
        isLoading={runningInstance()?.isLoading}
        onPlay={() => handleInstanceLaunch(props.instance)}
        onOpenFolder={() => handleOpenFolder(props.instance)}
        onRemove={() => setInstanceToRemove(props.instance)}
      >
        <ContextMenuTrigger>
          <InstanceCard
            isRunning={runningInstance()?.isRunning}
            isLoading={runningInstance()?.isLoading}
            onClick={goToInstancePage}
            onLaunchClick={(e) => {
              preventAll(e);
              handleInstanceLaunch(props.instance);
            }}
            onStopClick={(e) => {
              preventAll(e);
              handleInstanceStop(props.instance);
            }}
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
