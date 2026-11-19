import { A } from '@solidjs/router';
import { Show, type Component } from 'solid-js';

import type { ContentVersion } from '@/features/content';

import {
  createContentPageHref,
  useContent,
  useContentContext,
} from '@/features/content';
import { useTranslation } from '@/shared/i18n';
import { Badge } from '@/shared/ui-kit';

export type ContentVersionDependencyItemProps = {
  dependency: ContentVersion['dependencies'][number];
};

/** One dependency: its project's name and link when the provider names the project. */
export const ContentVersionDependencyItem: Component<
  ContentVersionDependencyItemProps
> = (props) => {
  const [{ t }] = useTranslation();
  const [context] = useContentContext();

  const project = useContent(() =>
    props.dependency.contentId && context.providerId
      ? {
          contentId: props.dependency.contentId,
          providerId: context.providerId,
        }
      : undefined,
  );

  const label = () =>
    project.data?.name ??
    props.dependency.fileName ??
    props.dependency.contentId ??
    props.dependency.versionId ??
    t('common.unknown');

  const href = () =>
    props.dependency.contentId
      ? createContentPageHref(props.dependency.contentId, {
          instanceId: context.instanceId,
          providerId: context.providerId,
        })
      : undefined;

  return (
    <li class='gap-2 flex items-center justify-between'>
      <Show
        when={href()}
        fallback={<span class='min-w-0 truncate'>{label()}</span>}
      >
        {(link) => (
          <A href={link()} class='min-w-0 truncate hover:underline'>
            {label()}
          </A>
        )}
      </Show>
      <Badge variant='secondary'>
        {t(`contentVersion.dependencyTypes.${props.dependency.dependencyType}`)}
      </Badge>
    </li>
  );
};
