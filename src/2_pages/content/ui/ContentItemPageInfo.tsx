import type { Component, ComponentProps } from 'solid-js';

import MdiDownloadIcon from '~icons/mdi/download';
import MdiOpenInNewIcon from '~icons/mdi/open-in-new';
import { splitProps } from 'solid-js';

import type { ContentItem } from '@/entities/instances';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Button, CombinedTooltip, IconButton, Image } from '@/shared/ui';

export type ContentItemPageInfoProps = {
  item: ContentItem;
  contentPageHref?: string;
};

export const ContentItemPageInfo: Component<
  ComponentProps<'div'> & ContentItemPageInfoProps
> = (props) => {
  const [local, others] = splitProps(props, [
    'item',
    'contentPageHref',
    'class',
  ]);

  const [{ t }] = useTranslation();

  return (
    <div class={cn('flex gap-2', local.class)} {...others}>
      <Image
        class='aspect-square size-24'
        src={local.item.iconUrl || undefined}
      />
      <div class='flex grow flex-col'>
        <div class='flex grow justify-between'>
          <div class='flex max-w-[512px] flex-col text-muted-foreground'>
            <span class='text-lg font-bold text-foreground'>
              <h1 class='text-2xl'>{local.item.name}</h1>
              &#32;
            </span>
            <span>{local.item.description}</span>
          </div>
          <div class='flex gap-2'>
            <Button>
              <MdiDownloadIcon />
              {t('common.install')}
            </Button>

            <CombinedTooltip
              label={t('content.openInBrowser')}
              as='div'
              class='size-min'
            >
              <IconButton
                variant='secondary'
                as='a'
                href={props.item.url}
                target='_blank'
                rel='noreferrer'
                icon={MdiOpenInNewIcon}
              />
            </CombinedTooltip>
          </div>
        </div>
      </div>
    </div>
  );
};
