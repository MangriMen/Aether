import MdiDownloadIcon from '~icons/mdi/download';
import IconMdiOpenInNew from '~icons/mdi/open-in-new';
import {
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import type { ContentType, ContentVersion } from '@/entities/instances';

import { cn } from '@/shared/lib';
import { IconButton } from '@/shared/ui';

import { useContentContext } from '../model';

export type ContentVersionActionsProps = ComponentProps<'div'> & {
  version: ContentVersion;
  slug?: string;
  contentType?: ContentType;
};

export const ContentVersionActions: Component<ContentVersionActionsProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'version',
    'slug',
    'contentType',
    'class',
  ]);

  const [, { installContent }] = useContentContext();

  const handleInstall = () => {
    if (!local.contentType) {
      return;
    }

    installContent(
      {
        id: local.version.contentId,
        contentType: local.contentType,
      },
      undefined,
      local.version.id,
    );
  };

  return (
    <div class={cn('flex gap-1', local.class)} {...others}>
      <IconButton
        class='bg-transparent group-hover:bg-primary'
        icon={MdiDownloadIcon}
        onClick={handleInstall}
      />
      <Show when={local.version.webUrl}>
        {(href) => (
          <IconButton
            as='a'
            class='bg-transparent'
            variant='secondary'
            icon={IconMdiOpenInNew}
            href={href()}
            target='_blank'
            rel='noreferrer'
          />
        )}
      </Show>
    </div>
  );
};
