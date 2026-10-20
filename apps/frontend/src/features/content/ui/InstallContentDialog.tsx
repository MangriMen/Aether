import type { DialogRootProps } from '@kobalte/core/dialog';

import IconMdiMagnify from '~icons/mdi/magnify';
import {
  createMemo,
  createSignal,
  For,
  Match,
  Show,
  splitProps,
  Switch,
  type Component,
} from 'solid-js';

import type { PartialBy } from '@/shared/utils';

import { useInstances } from '@/features/instance';
import { useTranslation } from '@/shared/i18n';
import {
  CombinedTextField,
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  LoadError,
  SkeletonList,
} from '@/shared/ui-kit';

import type { ContentCompatibilityCheckParams } from '../model/compatibility';
import type { ContentItem } from '../model/contentItem';
import type { ContentManager } from '../model/contentManager';

import { useCheckCompatibility } from '../api/contentQueries';
import { InstallContentDialogListItem } from './InstallContentDialogListItem';

export type InstallContentDialogProps = DialogRootProps & {
  item: ContentItem;
  manager: ContentManager;
};

export const InstallContentDialog: Component<InstallContentDialogProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['item', 'manager']);

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

  return (
    <Dialog {...others}>
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
              <div class='px-2.5 my-px flex h-full items-center'>
                <IconMdiMagnify />
              </div>
            }
          />
          <div class='gap-2 flex flex-col'>
            <Switch>
              <Match when={instances.isLoading}>
                <SkeletonList itemsCount={3} height={60} radius={6} />
              </Match>
              <Match when={instances.isError && !instances.data}>
                <LoadError
                  message={t('common.loadFailed')}
                  retryLabel={t('common.retry')}
                  onRetry={() => void instances.refetch()}
                />
              </Match>
              <Match when={(filteredInstances()?.length ?? 0) > 0}>
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
                      onCloseDialog={() => others.onOpenChange?.(false)}
                    />
                  )}
                </For>
              </Match>
              <Match when={filteredInstances()}>
                <span class='text-muted-foreground self-center'>
                  {t('content.noMatchingInstances')}
                </span>
              </Match>
            </Switch>
          </div>
        </Show>
      </DialogContent>
    </Dialog>
  );
};
