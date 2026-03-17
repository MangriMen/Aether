import type { Accessor } from 'solid-js';

import { For, type Component } from 'solid-js';

import type { ContentItem, Instance } from '@/entities/instances';

import {
  ContentInstallButton,
  useCheckCompatibility,
  useInstances,
} from '@/entities/instances';
import { useTranslation } from '@/shared/model';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/shared/ui';

export type InstallContentDialogProps = {
  provider: string;
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

  const checkCompatibility = useCheckCompatibility(
    () => instances.data?.map((instance) => instance.id) ?? [],
    () => ({
      provider: props.provider,
      contentItem: props.item,
    }),
  );

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

              const isCompatible = () =>
                checkCompatibility.data?.[instance.id].isCompatible ?? false;

              return (
                <div class='flex items-center justify-between rounded border p-2'>
                  <span>{instance.name}</span>
                  <ContentInstallButton
                    isInstalled={isInstalled()}
                    isInstalling={isInstalling()}
                    onClick={handleInstall}
                    isCompatible={isCompatible()}
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
