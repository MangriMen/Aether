import { Show } from 'solid-js';

import type { ContentFile } from '@/entities/instances';

import {
  getContentFileAuthor,
  isContentFileAuthorPending,
  getContentFileDisplayTitle,
  getContentFileIconSrc,
  isLocalContentFile,
  InstanceIcon,
} from '@/entities/instances';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Skeleton } from '@/shared/ui';

interface ContentNameCellProps {
  file: ContentFile;
}

export const ContentNameCell = (props: ContentNameCellProps) => {
  const [{ t }] = useTranslation();

  const subtitle = () =>
    isLocalContentFile(props.file)
      ? t('common.local')
      : getContentFileAuthor(props.file);

  return (
    <span
      class={cn('inline-flex items-center gap-2', {
        'text-muted-foreground': props.file.disabled,
      })}
    >
      <InstanceIcon class='size-8' src={getContentFileIconSrc(props.file)} />
      <span class='inline-flex flex-col'>
        <span>{getContentFileDisplayTitle(props.file)}</span>
        <Show
          when={subtitle()}
          fallback={
            <Show when={isContentFileAuthorPending(props.file)}>
              <Skeleton width={80} height={16} radius={4} />
            </Show>
          }
        >
          {(text) => <span class='text-muted-foreground'>{text()}</span>}
        </Show>
      </span>
    </span>
  );
};
