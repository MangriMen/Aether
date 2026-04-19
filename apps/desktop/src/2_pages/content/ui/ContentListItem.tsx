import { useNavigate } from '@solidjs/router';
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

  const navigate = useNavigate();

  const handleClick = () => {
    const href = contentPageHref();

    if (!href) {
      return;
    }

    navigate(href);
  };

  return (
    // TODO: add accessibility
    // eslint-disable-next-line jsx-a11y/click-events-have-key-events
    <div
      class={cn(
        'flex gap-2 border bg-card/card rounded-lg p-3 group',
        local.class,
      )}
      role='button'
      tabIndex={0}
      onClick={contentPageHref() ? handleClick : undefined}
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
