import { useParams } from '@solidjs/router';
import {
  createMemo,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { useContentContext, useContentVersions } from '@/features/content';
import { cn } from '@/shared/utils';

import type { ContentItemPageTabProps } from '../model/contentItemPageTabs';

import { ContentVersionsTable } from './ContentVersionsTable';
import { ContentVersionView } from './ContentVersionView';

export type ContentVersionsTabProps = ComponentProps<'div'> &
  ContentItemPageTabProps;

export const ContentVersionsTab: Component<ContentVersionsTabProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['item', 'class']);

  const [context] = useContentContext();
  const params = useParams();

  const versions = useContentVersions(() =>
    local.item?.id && context.providerId
      ? {
          contentId: local.item?.id,
          providerId: context.providerId,
        }
      : undefined,
  );

  const versionsData = createMemo(() => versions.data ?? []);

  return (
    <div class={cn('flex flex-col', local.class)} {...others}>
      {/* Hidden, not unmounted, while a version is open: Back keeps the page and the sort. */}
      <ContentVersionsTable
        class={cn({ hidden: Boolean(params.versionId) })}
        data={versionsData()}
        contentType={local.item?.contentType}
        isLoading={versions.isFetching}
      />
      <Show when={params.versionId}>
        {(versionId) => (
          <ContentVersionView
            class='min-h-0 flex-1 overflow-auto'
            versionId={versionId()}
            item={local.item}
          />
        )}
      </Show>
    </div>
  );
};
