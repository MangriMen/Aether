import { useParams } from '@solidjs/router';
import {
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { useContent, useContentVersion } from '@/entities/instances';
import { cn } from '@/shared/lib';
import { Separator } from '@/shared/ui';

import { useContentContext } from '../model';
import { ContentItemPageInfo } from './ContentItemPageInfo';
import { ContentVersionsTable } from './ContentVersionsTable';

export type ContentItemPageProps = ComponentProps<'div'>;

export const ContentItemPage: Component<ContentItemPageProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [context] = useContentContext();

  const params = useParams();

  const item = useContent(() =>
    params.contentId && context.providerId
      ? {
          contentId: params.contentId,
          providerId: context.providerId,
        }
      : undefined,
  );

  const versions = useContentVersion(() =>
    params.contentId && context.providerId
      ? {
          contentId: params.contentId,
          providerId: context.providerId,
        }
      : undefined,
  );

  return (
    <div
      class={cn('flex flex-col grow gap-2 overflow-hidden', local.class)}
      {...others}
    >
      <Show when={item.data}>
        {(item) => (
          <>
            <ContentItemPageInfo item={item()} />
            <Separator />
          </>
        )}
      </Show>

      <ContentVersionsTable
        data={versions.data ?? []}
        slug={item.data?.slug}
        contentType={item.data?.contentType}
        isLoading={versions.isFetching}
      />
    </div>
  );
};
