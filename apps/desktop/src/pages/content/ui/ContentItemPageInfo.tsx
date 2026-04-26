import type { Component, ComponentProps } from 'solid-js';

import MdiDownloadIcon from '~icons/mdi/download';
import MdiOpenInNewIcon from '~icons/mdi/open-in-new';
import { createMemo, splitProps } from 'solid-js';

import type { ContentItem } from '../../../entities/instances';

import { cn } from '../../../shared/lib';
import { useTranslation } from '../../../shared/model';
import {
  Button,
  CombinedTooltip,
  DelayedShow,
  IconButton,
  Image,
  Skeleton,
} from '../../../shared/ui';
import { useContentContext } from '../model';

export type ContentItemPageInfoProps = {
  item: ContentItem | undefined;
  contentPageHref?: string;
  isLoading?: boolean;
};

export const ContentItemPageInfo: Component<
  ComponentProps<'div'> & ContentItemPageInfoProps
> = (props) => {
  const [local, others] = splitProps(props, [
    'item',
    'contentPageHref',
    'isLoading',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const [context, { createIsInstalled, createIsInstalling }] =
    useContentContext();

  const isInstalledMemo = createMemo(() =>
    createIsInstalled(
      () => local.item?.id,
      () => context.instanceId,
    ),
  );
  const isInstalled = () => isInstalledMemo()();

  const isInstallingMemo = createMemo(() =>
    createIsInstalling(() => local.item?.id),
  );
  const isInstalling = () => isInstallingMemo()();

  const itemIsLoading = createMemo(() => !local.item && local.isLoading);

  return (
    <div class={cn('flex gap-3', local.class)} {...others}>
      <Image
        class='aspect-square size-24'
        src={local.item?.iconUrl || undefined}
      />
      <div class='flex grow flex-col'>
        <div class='flex grow justify-between'>
          <div class='flex max-w-[512px] flex-col'>
            <h1 class='text-2xl font-bold text-foreground'>
              <DelayedShow
                when={!itemIsLoading()}
                fallback={<Skeleton width={256} height={32} />}
              >
                {local.item?.name}
              </DelayedShow>
            </h1>
            <span class='text-muted-foreground'>
              <DelayedShow
                when={!itemIsLoading()}
                fallback={<Skeleton width={256} height={32} />}
              >
                {local.item?.description}
              </DelayedShow>
            </span>
          </div>
          <div class='flex gap-2'>
            <CombinedTooltip
              label={t('content.alreadyInstalled')}
              disableTooltip={!isInstalled()}
              as={Button}
              loading={itemIsLoading() || isInstalling()}
              disabled={isInstalled()}
            >
              <MdiDownloadIcon />
              {t('common.install')}
            </CombinedTooltip>

            <CombinedTooltip
              label={t('content.openInBrowser')}
              as='div'
              class='size-min'
            >
              <IconButton
                variant='secondary'
                as='a'
                href={props.item?.url}
                target='_blank'
                rel='noreferrer'
                icon={MdiOpenInNewIcon}
                loading={itemIsLoading()}
              />
            </CombinedTooltip>
          </div>
        </div>
      </div>
    </div>
  );
};
