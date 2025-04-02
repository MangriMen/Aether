import {
  installInstance,
  InstanceInstallStage,
  updateInstance,
  type InstanceSettingsTabProps,
} from '@/entities/instances';
import { cn } from '@/shared/lib';
import { Button, Image, LabeledField, showToast } from '@/shared/ui';
import {
  createMemo,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';
import MdiHammerIcon from '@iconify/icons-mdi/hammer';
import { isAetherLauncherError } from '@/shared/model';

export type InstallationTabProps = ComponentProps<'div'> &
  InstanceSettingsTabProps;

export const InstallationTab: Component<InstallationTabProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const handleUpdate = async () => {
    try {
      await updateInstance(local.instance.id);
    } catch (e) {
      if (isAetherLauncherError(e)) {
        showToast({
          title: 'Failed to update instance',
          variant: 'destructive',
          description: e.message,
        });
      }
    }
  };

  const handleRepair = async () => {
    try {
      await installInstance(local.instance.id, true);
    } catch (e) {
      if (isAetherLauncherError(e)) {
        showToast({
          title: 'Failed to repair instance',
          variant: 'destructive',
          description: e.message,
        });
      }
    }
  };

  const isInstalling = createMemo(
    () => local.instance.installStage !== InstanceInstallStage.Installed,
  );

  return (
    <div class={cn('flex flex-col gap-2', local.class)} {...others}>
      <LabeledField label='Currently installed'>
        <div class='flex items-center gap-3 rounded-lg bg-background p-3 text-muted-foreground'>
          <Image class='h-full w-max p-0' />
          <div class='flex flex-col'>
            <div class='text-base font-medium'>
              Minecraft {local.instance.gameVersion}
            </div>
            <div class='capitalize'>{local.instance.loader}</div>
            <Show when={local.instance.packInfo}>
              <div>Managed by: {local.instance.packInfo?.packType}</div>
            </Show>
          </div>
          <div class='ml-auto flex items-center gap-1'>
            <Show when={local.instance.packInfo}>
              <Button
                class='text-base'
                size='sm'
                variant='ghost'
                onClick={handleUpdate}
                disabled={isInstalling()}
              >
                Update
              </Button>
            </Show>
            <Button
              class='text-base'
              size='sm'
              variant='ghostWarning'
              leadingIcon={MdiHammerIcon}
              onClick={handleRepair}
              disabled={isInstalling()}
            >
              Repair
            </Button>
          </div>
        </div>
      </LabeledField>
    </div>
  );
};
