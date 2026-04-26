import IconMdiMagnify from '~icons/mdi/magnify';
import {
  createMemo,
  createSignal,
  For,
  Show,
  splitProps,
  type Component,
} from 'solid-js';

import type { ContentItem } from '../../../entities/instances';
import type { ContentCompatibilityCheckParams } from '../../../entities/instances/model/compatibility';
import type { PartialBy } from '../../../shared/model';
import type { ContentManager } from '../model';

import {
  useCheckCompatibility,
  useInstances,
} from '../../../entities/instances';
import { useTranslation } from '../../../shared/model';
import {
  CombinedTextField,
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '../../../shared/ui';
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

  const contentCheckParams = (): PartialBy<
    ContentCompatibilityCheckParams,
    'providerId'
  > => ({
    providerId: local.manager.providerId(),
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

  const handleOnOpenChange = (isOpen: boolean) => {
    if (!isOpen) {
      local.onClose();
    }
  };

  return (
    <Dialog defaultOpen onOpenChange={handleOnOpenChange}>
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
            <Show
              when={filteredInstances.length <= 0}
              fallback={
                <span class='self-center text-muted-foreground'>
                  {t('content.noMatchingInstances')}
                </span>
              }
            >
              <For each={filteredInstances()}>
                {(instance) => (
                  <InstallContentDialogListItem
                    item={props.item}
                    instance={instance}
                    manager={local.manager}
                    checkCompatibilityData={checkCompatibility.data}
                    isLoadingCheckCompatibilityData={
                      checkCompatibility.isLoading
                    }
                    onCloseDialog={local.onClose}
                  />
                )}
              </For>
            </Show>
          </div>
        </Show>
      </DialogContent>
    </Dialog>
  );
};
