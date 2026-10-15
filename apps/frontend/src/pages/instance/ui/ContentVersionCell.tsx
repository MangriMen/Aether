import { A, useParams } from '@solidjs/router';
import { Match, Show, Switch } from 'solid-js';

import type { ContentFile } from '@/entities/instances';

import {
  getContentFileProjectHref,
  getContentFileVersionNumber,
  getContentFileVersionState,
} from '@/entities/instances';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

interface ContentVersionCellProps {
  file: ContentFile;
}

export const ContentVersionCell = (props: ContentVersionCellProps) => {
  const [{ t }] = useTranslation();
  const params = useParams();

  const href = () => getContentFileProjectHref(props.file, params.id);

  return (
    <span
      class={cn('inline-flex flex-col', {
        'text-muted-foreground': props.file.disabled,
      })}
    >
      <Switch>
        <Match when={getContentFileVersionState(props.file) === 'known'}>
          <Show
            when={href()}
            fallback={<span>{getContentFileVersionNumber(props.file)}</span>}
          >
            {(link) => (
              <A href={link()} class='hover:underline'>
                {getContentFileVersionNumber(props.file)}
              </A>
            )}
          </Show>
        </Match>
        <Match when={getContentFileVersionState(props.file) === 'pending'}>
          <span class='text-muted-foreground'>{t('common.loading')}</span>
        </Match>
        <Match when>
          <span>{t('common.unknown')}</span>
        </Match>
      </Switch>
      <span class='text-muted-foreground'>{props.file.fileName}</span>
    </span>
  );
};
