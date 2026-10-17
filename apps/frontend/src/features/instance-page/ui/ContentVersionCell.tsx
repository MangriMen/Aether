import { A, useParams } from '@solidjs/router';
import { createMemo, Match, Show, Switch } from 'solid-js';

import type { ContentFile } from '@/features/content';

import {
  getContentFileProjectHref,
  getContentFileVersionNumber,
  getContentFileVersionState,
} from '@/features/content';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

interface ContentVersionCellProps {
  file: ContentFile;
}

export const ContentVersionCell = (props: ContentVersionCellProps) => {
  const [{ t }] = useTranslation();
  const params = useParams();

  const href = () => getContentFileProjectHref(props.file, params.id);

  const state = createMemo(() => getContentFileVersionState(props.file));
  const versionNumber = () => getContentFileVersionNumber(props.file);

  return (
    <span
      class={cn('inline-flex flex-col', {
        'text-muted-foreground': props.file.disabled,
      })}
    >
      <Switch>
        <Match when={state() === 'known'}>
          <Show when={href()} fallback={<span>{versionNumber()}</span>}>
            {(link) => (
              <A href={link()} class='hover:underline'>
                {versionNumber()}
              </A>
            )}
          </Show>
        </Match>
        <Match when={state() === 'pending'}>
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
