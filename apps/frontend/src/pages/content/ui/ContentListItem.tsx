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
        'group gap-2 rounded-md bg-card/card p-3 relative flex border',
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
          isLoading={isLoading()}
          onClick={handleInstall}
        />
      </div>
    </div>
  );
};
