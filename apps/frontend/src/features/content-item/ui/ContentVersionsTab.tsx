import {
  createMemo,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { useContentContext, useContentVersions } from '@/features/content';
import { cn } from '@/shared/utils';

import type { ContentItemPageTabProps } from '../model/contentItemPageTabs';

import { ContentVersionsTable } from './ContentVersionsTable';

export type ContentVersionsTabProps = ComponentProps<'div'> &
  ContentItemPageTabProps;

export const ContentVersionsTab: Component<ContentVersionsTabProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['item', 'class']);

  const [context] = useContentContext();

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
      <ContentVersionsTable
        data={versionsData()}
        contentType={local.item?.contentType}
        isLoading={versions.isFetching}
      />
    </div>
  );
};
