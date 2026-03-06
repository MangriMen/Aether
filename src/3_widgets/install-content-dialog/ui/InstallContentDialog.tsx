import type { Accessor } from 'solid-js';

import { For, type Component } from 'solid-js';

import type { ContentItem, Instance } from '@/entities/instances';

import { ContentInstallButton, useInstances } from '@/entities/instances';
import { useTranslation } from '@/shared/model';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/shared/ui';

export type InstallContentDialogProps = {
  item: ContentItem;
  onClose: () => void;
  installContent: (
    item: ContentItem,
    instanceId?: Instance['id'],
  ) => Promise<void>;
  createIsContentInstalling: (
    contentId: Accessor<string | undefined>,
    instanceId: Accessor<string | undefined>,
  ) => Accessor<boolean>;
  createIsContentInstalled: (
    contentId: Accessor<string | undefined>,
    instanceId: Accessor<string | undefined>,
  ) => Accessor<boolean>;
};

export const InstallContentDialog: Component<InstallContentDialogProps> = (
  props,
) => {
  const [{ t }] = useTranslation();
  const instances = useInstances();

  return (
    <Dialog defaultOpen={true}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('instance.installContent')}</DialogTitle>
        </DialogHeader>
        <div class='flex flex-col gap-2'>
          <For each={instances.data}>
            {(instance) => {
              const isInstalled = props.createIsContentInstalled(
                () => props.item.providerData?.id as string,
                () => instance.id,
              );

              const isInstalling = props.createIsContentInstalling(
                () => props.item.providerData?.id as string,
                () => instance.id,
              );

              const handleInstall = () =>
                props.installContent(props.item, instance.id);

              return (
                <div class='flex items-center justify-between rounded border p-2'>
                  <span>{instance.name}</span>
                  <ContentInstallButton
                    isInstalled={isInstalled()}
                    isInstalling={isInstalling()}
                    onClick={handleInstall}
                  />
                </div>
              );
            }}
          </For>
        </div>
      </DialogContent>
    </Dialog>
  );
};
