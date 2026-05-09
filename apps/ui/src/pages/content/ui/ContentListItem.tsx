import { A } from '@solidjs/router';
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

  const handleInstall = (e: MouseEvent) => {
    e.stopPropagation();
    requestInstall();
  };

  return (
    <div
      class={cn(
        'relative flex gap-2 border bg-card/card rounded-lg p-3 group',
        local.class,
      )}
      {...others}
    >
      <A
        href={contentPageHref()}
        class='after:absolute after:inset-0 after:z-10'
      >
        <ContentItemInfo item={local.item} />
      </A>
      <div class='relative z-20 ml-auto mt-auto flex size-max flex-col justify-end'>
        <ContentInstallButton
          isInstalling={isInstalling()}
          isInstalled={isInstalled()}
          isLoading={isLoading()}
          onClick={handleInstall}
        />
      </div>
    </div>
  );
};
