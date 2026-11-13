import IconMdiOpenInNew from '~icons/mdi/open-in-new';
import {
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import type { ContentType, ContentVersion } from '@/features/content';

import { useTranslation } from '@/shared/i18n';
import { IconButton } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import { useContentVersionInstall } from '../lib/useContentVersionInstall';

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

  const [{ t }] = useTranslation();

  const { install, icon, isCurrentVersionInstalled, isSwap, isInstalling } =
    useContentVersionInstall(
      () => local.version,
      () => local.contentType,
    );

  const installLabel = () => {
    if (isCurrentVersionInstalled()) {
      return t('common.installed');
    }

    return isSwap() ? t('common.swapVersion') : t('common.install');
  };

  return (
    <div class={cn('gap-1 flex', local.class)} {...others}>
      <IconButton
        class='
          enabled:group-hover:bg-primary
          enabled:group-hover:hover:bg-primary/solid-hover
          bg-transparent
        '
        icon={icon()}
        aria-label={installLabel()}
        onClick={install}
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
            aria-label={t('content.openInBrowser')}
            href={href()}
            target='_blank'
            rel='noreferrer'
          />
        )}
      </Show>
    </div>
  );
};
