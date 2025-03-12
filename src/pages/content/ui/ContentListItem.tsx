import type { ContentItem } from '@/entities/instances';
import { cn } from '@/shared/lib';
import { Button, Image } from '@/shared/ui';
import { splitProps, type Component, type ComponentProps } from 'solid-js';
import MdiDownload from '@iconify/icons-mdi/download';
import { useTranslate } from '@/shared/model';

export type ContentListItemProps = ComponentProps<'div'> & {
  item: ContentItem;
  onInstall: () => void;
};

export const ContentListItem: Component<ContentListItemProps> = (props) => {
  const [local, others] = splitProps(props, ['item', 'onInstall', 'class']);

  const [{ t }] = useTranslate();

  return (
    <div
      class={cn(
        'flex gap-2 border border-secondary rounded-lg p-3',
        local.class,
      )}
      {...others}
    >
      <Image
        class='aspect-square size-24'
        src={local.item.iconUrl || undefined}
      />
      <div class='flex flex-col text-muted-foreground'>
        <span class='text-lg font-bold text-foreground'>
          <a href={local.item.url} target='_blank'>
            {local.item.name}
          </a>{' '}
          <span class='text-base font-semibold text-muted-foreground'>
            by {local.item.author}
          </span>
        </span>
        <span>{local.item.description}</span>
      </div>
      <div class='ml-auto flex flex-col justify-end'>
        <Button
          class='px-3'
          variant='outline'
          leadingIcon={MdiDownload}
          onClick={local.onInstall}
        >
          {t('common.install')}
        </Button>
      </div>
    </div>
  );
};
