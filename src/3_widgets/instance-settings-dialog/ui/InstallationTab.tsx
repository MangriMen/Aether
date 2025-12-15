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
import { useTranslation } from '@/shared/model';
import { Button, Image, LabeledField } from '@/shared/ui';

import type { InstanceSettingsTabProps } from '../model';

export type InstallationTabProps = ComponentProps<'div'> &
  InstanceSettingsTabProps;

export const InstallationTab: Component<InstallationTabProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const [{ t }] = useTranslation();

  const { mutateAsync: installInstance } = useInstallInstance();
  const { mutateAsync: updateInstance } = useUpdateInstance();

  const handleUpdate = async () => {
    await updateInstance(local.instance.id);
  };

  const handleRepair = async () => {
    await installInstance({ id: local.instance.id, force: true });
  };

  const isInstalling = createMemo(
    () => local.instance.installStage !== InstanceInstallStage.Installed,
  );

  return (
    <div class={cn('flex flex-col gap-2', local.class)} {...others}>
      <LabeledField
        label={
          <span class='text-lg font-medium'>
            {t('instance.instanceSettings.currentlyInstalled')}
          </span>
        }
      >
        <div class='flex items-center gap-3 rounded-lg bg-background p-3 text-muted-foreground'>
          <Image class='size-12 p-1' />
          <div class='flex flex-col'>
            <div class='text-base font-medium'>
              Minecraft {local.instance.gameVersion}
            </div>
            <div class='capitalize'>{local.instance.loader}</div>
          </div>
          <div class='ml-auto flex items-center gap-1'>
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
      </LabeledField>

      <Show when={local.instance.packInfo}>
        <LabeledField
          label={
            <span class='text-lg font-medium'>
              {t('instance.instanceSettings.modpack')}
            </span>
          }
        >
          <div class='flex items-center gap-3 rounded-lg bg-background p-3 text-muted-foreground'>
            <Image class='size-12 p-1' />

            <div class='flex flex-col'>
              {/* TODO: add modpack name to packInfo */}
              <span class='text-base font-medium capitalize'>
                {local.instance.packInfo?.modpackId}
              </span>
              <span>{local.instance.packInfo?.version}</span>
            </div>
            <div class='ml-auto flex items-center gap-1'>
              <Button
                size='sm'
                variant='ghost'
                onClick={handleUpdate}
                disabled={isInstalling()}
              >
                {t('common.update')}
              </Button>
            </div>
          </div>
        </LabeledField>
      </Show>
    </div>
  );
};
