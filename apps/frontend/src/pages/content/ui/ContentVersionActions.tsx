import IconMdiCheck from '~icons/mdi/check';
import IconMdiDownload from '~icons/mdi/download';
import IconMdiOpenInNew from '~icons/mdi/open-in-new';
import IconMdiSwapVertical from '~icons/mdi/swap-vertical';
import {
  createMemo,
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
  contentType?: ContentType;
};

export const ContentVersionActions: Component<ContentVersionActionsProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'version',
    'contentType',
    'class',
  ]);

  const [
    context,
    { installContent, createInstalledVersion, createIsInstalling },
  ] = useContentContext();

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

  const installedVersionMemo = createMemo(() =>
    createInstalledVersion(
      () => local.version.contentId,
      () => context.instanceId,
    ),
  );

  const installedVersion = () => installedVersionMemo()();

  const isCurrentVersionInstalled = createMemo(
    () => installedVersion() === local.version.id,
  );

  const installIcon = createMemo(() => {
    if (isCurrentVersionInstalled()) {
      return IconMdiCheck;
    }

    return installedVersion() ? IconMdiSwapVertical : IconMdiDownload;
  });

  const isInstallingMemo = createMemo(() =>
    createIsInstalling(() => local.version.contentId),
  );
  const isInstalling = () => isInstallingMemo()();

  return (
    <div class={cn('gap-1 flex', local.class)} {...others}>
      <IconButton
        class='
          enabled:group-hover:bg-primary
          enabled:group-hover:hover:bg-primary/solid-hover
          bg-transparent
        '
        icon={installIcon()}
        onClick={handleInstall}
        loading={isInstalling()}
        disabled={isCurrentVersionInstalled()}
      />
      <Show when={local.version.webUrl}>
        {(href) => (
          <IconButton
            as='a'
            class='
              group-hover:hover:bg-secondary/hover
              bg-transparent
            '
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
