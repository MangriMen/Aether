import { Show } from 'solid-js';

import type { ContentFile } from '@/features/content';

import {
  getContentFileAuthor,
  isContentFileAuthorPending,
  getContentFileDisplayTitle,
  getContentFileIconSrc,
  isLocalContentFile,
} from '@/features/content';
import { InstanceIcon } from '@/features/instance';
import { useTranslation } from '@/shared/i18n';
import { Skeleton } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

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
