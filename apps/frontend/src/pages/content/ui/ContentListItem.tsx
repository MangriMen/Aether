import { A } from '@solidjs/router';
import {
  createMemo,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { ContentInstallButton, type ContentItem } from '@/entities/instances';
import { cn } from '@/shared/lib';

import { createContentPageHref, useContentItemActions } from '../lib';
import { useContentContext } from '../model';
import { ContentItemInfo } from './ContentItemInfo';

export type ContentListItemProps = ComponentProps<'div'> & {
  item: ContentItem;
};

export const ContentListItem: Component<ContentListItemProps> = (props) => {
  const [local, others] = splitProps(props, ['item', 'class']);

  const [context] = useContentContext();

  const { isInstalled, isInstalling, isPrefetching, requestInstall } =
    useContentItemActions(() => local.item);

  const contentPageHref = createMemo(() =>
    createContentPageHref(local.item.id, {
      instanceId: context.instanceId,
      providerId: context.providerId,
    }),
  );

  const handleInstall = (e: MouseEvent) => {
    e.stopPropagation();
    requestInstall();
  };

  return (
    <div
      class={cn(
        'group gap-2 rounded-md bg-card/card hover:bg-card/hover p-3 relative flex border',
        local.class,
      )}
      {...others}
    >
      <A
        href={contentPageHref()}
        class='after:inset-0 after:absolute after:z-10'
      >
        <ContentItemInfo item={local.item} />
      </A>
      <div
        class='
        relative z-20 mt-auto ml-auto flex size-max flex-col justify-end
      '
      >
        <ContentInstallButton
          isInstalling={isInstalling()}
          isInstalled={isInstalled()}
          isLoading={isPrefetching()}
          onClick={handleInstall}
        />
      </div>
    </div>
  );
};
