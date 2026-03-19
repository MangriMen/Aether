import IconMdiMagnify from '~icons/mdi/magnify';
import {
  createMemo,
  createSignal,
  For,
  Show,
  splitProps,
  type Component,
} from 'solid-js';

import type { ContentItem } from '@/entities/instances';

import { useCheckCompatibility, useInstances } from '@/entities/instances';
import { useTranslation } from '@/shared/model';
import {
  CombinedTextField,
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/shared/ui';

import type { ContentManager } from '../model';

import { InstallContentDialogListItem } from './InstallContentDialogListItem';

export type InstallContentDialogProps = {
  item: ContentItem;
  manager: ContentManager;
  onClose: () => void;
};

export const InstallContentDialog: Component<InstallContentDialogProps> = (
  props,
) => {
  const [local, _] = splitProps(props, ['item', 'manager', 'onClose']);

  const [{ t }] = useTranslation();
  const instances = useInstances();

  const instanceIds = createMemo(
    () => instances.data?.map((instance) => instance.id) ?? [],
  );

  const contentCheckParams = () => ({
    provider: local.manager.providerId(),
    contentItem: props.item,
  });

  const checkCompatibility = useCheckCompatibility(
    () => instanceIds(),
    () => contentCheckParams(),
  );

  const [searchQuery, setSearchQuery] = createSignal('');

  const filteredInstances = createMemo(() =>
    instances.data?.filter((instance) =>
      instance.name.toLowerCase().includes(searchQuery().toLowerCase()),
    ),
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
                  item={props.item}
                  instance={instance}
                  manager={local.manager}
                  checkCompatibilityData={checkCompatibility.data}
                  isLoadingCheckCompatibilityData={checkCompatibility.isLoading}
                  onCloseDialog={local.onClose}
                />
              )}
            </For>
          </div>
        </Show>
      </DialogContent>
    </Dialog>
  );
};
