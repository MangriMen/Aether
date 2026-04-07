import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { ContentInstallButton, type ContentItem } from '@/entities/instances';
import { cn } from '@/shared/lib';

import { useContentListItem } from '../lib';
import { ContentItemInfo } from './ContentItemInfo';

export type ContentListItemProps = ComponentProps<'div'> & {
  item: ContentItem;
};

export const ContentListItem: Component<ContentListItemProps> = (props) => {
  const [local, others] = splitProps(props, ['item', 'class']);

  const {
    requestInstall,
    isInstalling,
    isInstalled,
    isLoading,
    contentPageHref,
  } = useContentListItem(() => local.item);

  return (
    <div
      class={cn(
        'flex gap-2 border border-secondary rounded-lg p-3',
        local.class,
      )}
      {...others}
    >
      <ContentItemInfo item={local.item} contentPageHref={contentPageHref()} />
      <div class='ml-auto flex flex-col justify-end'>
        <ContentInstallButton
          isInstalling={isInstalling()}
          isInstalled={isInstalled()}
          isLoading={isLoading()}
          onClick={requestInstall}
        />
      </div>
    </div>
  );
};
