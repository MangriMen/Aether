import IconMdiHammer from '~icons/mdi/hammer';
import {
  createMemo,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import {
  InstanceInstallStage,
  useInstallInstance,
  useUpdateInstance,
} from '@/entities/instances';
import { cn } from '@/shared/lib';
import { isLauncherError, useTranslation } from '@/shared/model';
import { Button, Image, LabeledField, showToast } from '@/shared/ui';

import type { InstanceSettingsTabProps } from '../model';

export type InstallationTabProps = ComponentProps<'div'> &
  InstanceSettingsTabProps;

export const InstallationTab: Component<InstallationTabProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const [{ t }] = useTranslation();

  const { mutateAsync: installInstance } = useInstallInstance();
  const { mutateAsync: updateInstance } = useUpdateInstance();

  const handleUpdate = async () => {
    try {
      await updateInstance(local.instance.id);
    } catch (e) {
      if (isLauncherError(e)) {
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
      await installInstance({ id: local.instance.id, force: true });
    } catch (e) {
      if (isLauncherError(e)) {
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
        <div class='flex flex-col gap-1 rounded-lg bg-background p-3 text-muted-foreground'>
          <div class='flex items-center gap-3'>
            <Image class='size-12 p-1' />
            <div class='flex flex-col'>
              <div class='text-base font-medium'>
                Minecraft {local.instance.gameVersion}
              </div>
              <div class='capitalize'>{local.instance.loader}</div>
            </div>
            <div class='ml-auto flex items-center gap-1'>
              <Show when={local.instance.packInfo}>
                <Button
                  size='sm'
                  variant='ghost'
                  onClick={handleUpdate}
                  disabled={isInstalling()}
                >
                  {t('common.update')}
                </Button>
              </Show>
              <Button
                size='sm'
                variant='ghostWarning'
                leadingIcon={IconMdiHammer}
                onClick={handleRepair}
                disabled={isInstalling()}
              >
                {t('common.repair')}
              </Button>
            </div>
          </div>
          <Show when={local.instance.packInfo}>
            <span>Modpack: {local.instance.packInfo?.packType}</span>
          </Show>
        </div>
      </LabeledField>
    </div>
  );
};
