import type { Accessor } from 'solid-js';

import IconMdiMagnify from '~icons/mdi/magnify';
import { createMemo, createSignal, For, Show, type Component } from 'solid-js';

import type { ContentItem, Instance } from '@/entities/instances';

import { useCheckCompatibility, useInstances } from '@/entities/instances';
import { useTranslation } from '@/shared/model';
import {
  CombinedTextField,
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/shared/ui';

import { InstallContentDialogListItem } from './InstallContentDialogListItem';

export type InstallContentDialogProps = {
  providerId?: string;
  providerDataContentIdField?: string;
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
      provider: props.providerId ?? '',
      contentItem: props.item,
    }),
  );

  const itemId = createMemo(() => {
    if (!props.item.providerData || !props.providerDataContentIdField) {
      return undefined;
    }

    const id = props.item.providerData[props.providerDataContentIdField];

    if (typeof id !== 'string') {
      return undefined;
    }

    return id;
  });

  const [searchQuery, setSearchQuery] = createSignal('');

  const filteredInstances = createMemo(() =>
    instances.data?.filter((instance) => instance.name.includes(searchQuery())),
  );

  return (
    <Dialog defaultOpen={true}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('instance.installContent')}</DialogTitle>
        </DialogHeader>
        <Show
          when={!checkCompatibility.isError}
          fallback={
            <div>{t('instance.unableToGetCompatibilityStatistics')}</div>
          }
        >
          <CombinedTextField
            value={searchQuery()}
            onChange={setSearchQuery}
            inputProps={{
              class: 'pl-10',
              placeholder: t('instance.searchInstance'),
              type: 'text',
            }}
            leadingIcons={
              <div class='my-px flex h-full items-center px-2.5'>
                <IconMdiMagnify />
              </div>
            }
          />
          <div class='flex flex-col gap-2'>
            <For each={filteredInstances()}>
              {(instance) => (
                <InstallContentDialogListItem
                  itemId={itemId()}
                  item={props.item}
                  instance={instance}
                  checkCompatibilityData={checkCompatibility.data}
                  isLoadingCheckCompatibilityData={checkCompatibility.isLoading}
                  installContent={props.installContent}
                  createIsContentInstalled={props.createIsContentInstalled}
                  createIsContentInstalling={props.createIsContentInstalling}
                />
              )}
            </For>
          </div>
        </Show>
      </DialogContent>
    </Dialog>
  );
};
