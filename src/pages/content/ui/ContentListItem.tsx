import type { ContentItem } from '@/entities/instances';
import { cn } from '@/shared/lib';
import { Button, Image } from '@/shared/ui';
import { splitProps, type Component, type ComponentProps } from 'solid-js';
import MdiDownload from '@iconify/icons-mdi/download';

export type ContentListItemProps = ComponentProps<'div'> & {
  item: ContentItem;
};

export const ContentListItem: Component<ContentListItemProps> = (props) => {
  const [local, others] = splitProps(props, ['item', 'class']);

  return (
    <div
      class={cn(
        'flex gap-2 border border-secondary rounded-lg p-3',
        local.class,
      )}
      {...others}
    >
      <Image class='h-24 w-max' />
      <div class='flex flex-col text-muted-foreground'>
        <span class='text-lg font-bold text-foreground'>
          {local.item.name}{' '}
          <span class='text-base font-semibold text-muted-foreground'>
            by {local.item.author}
          </span>
        </span>
        <span>{local.item.description}</span>
      </div>
      <div class='ml-auto flex flex-col justify-end'>
        <Button class='px-3' variant='outline' leadingIcon={MdiDownload}>
          Install
        </Button>
      </div>
    </div>
  );
};
