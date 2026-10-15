import { Show } from 'solid-js';

import type { ContentFile } from '@/entities/instances';

import {
  getContentFileAuthor,
  getContentFileDisplayTitle,
  getContentFileIconSrc,
  isLocalContentFile,
  InstanceIcon,
} from '@/entities/instances';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

interface ContentNameCellProps {
  file: ContentFile;
}

export const ContentNameCell = (props: ContentNameCellProps) => {
  const [{ t }] = useTranslation();

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
          when={!isLocalContentFile(props.file)}
          fallback={
            <span class='text-muted-foreground'>{t('common.local')}</span>
          }
        >
          <Show when={getContentFileAuthor(props.file)}>
            {(author) => <span class='text-muted-foreground'>{author()}</span>}
          </Show>
        </Show>
      </span>
    </span>
  );
};
