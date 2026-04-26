import {
  createMemo,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { useContentVersion } from '@/entities/instances';
import { cn } from '@/shared/lib';

import type { ContentItemPageTabProps } from '../../model';

import { useContentContext } from '../../model';
import { ContentVersionsTable } from '../ContentVersionsTable';

export type ContentVersionsTabProps = ComponentProps<'div'> &
  ContentItemPageTabProps;

export const ContentVersionsTab: Component<ContentVersionsTabProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['item', 'class']);

  const [context] = useContentContext();

  const versions = useContentVersion(() =>
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
