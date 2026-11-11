import {
  For,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import type { ContentVersion } from '@/features/content';

import { dayjs, formatBytes, formatTime } from '@/shared/format';
import { useTranslation } from '@/shared/i18n';
import { Badge } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import { ContentVersionBadgeList } from './ContentVersionBadgeList';
import { ContentVersionDependencyItem } from './ContentVersionDependencyItem';

export type ContentVersionDetailsProps = ComponentProps<'div'> & {
  version: ContentVersion;
};

const Section: Component<ComponentProps<'section'> & { title?: string }> = (
  props,
) => {
  const [local, others] = splitProps(props, ['title', 'children', 'class']);

  return (
    <section
      class={cn(
        'gap-2 p-4 flex flex-col rounded-md border bg-card/card',
        local.class,
      )}
      {...others}
    >
      <h2 class='text-base font-semibold'>{local.title}</h2>
      {local.children}
    </section>
  );
};

/** What the provider says of the version besides its changelog: for the side of the page. */
export const ContentVersionDetails: Component<ContentVersionDetailsProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['version', 'class']);

  const [{ t, locale }] = useTranslation();

  return (
    <div class={cn('gap-4 flex flex-col', local.class)} {...others}>
      <Section title={t('contentVersion.details')}>
        <dl class='gap-x-4 gap-y-2 grid grid-cols-[auto_1fr]'>
          <dt class='text-muted-foreground'>{t('contentVersion.channel')}</dt>
          <dd>{t(`contentVersion.channels.${local.version.versionType}`)}</dd>

          <dt class='text-muted-foreground'>{t('contentVersion.published')}</dt>
          <dd>{formatTime(dayjs(local.version.datePublished))}</dd>

          <Show when={local.version.gameVersions.length > 0}>
            <dt class='text-muted-foreground'>
              {t('contentVersion.gameVersion')}
            </dt>
            <dd>
              <ContentVersionBadgeList items={local.version.gameVersions} />
            </dd>
          </Show>

          <Show when={local.version.loaders.length > 0}>
            <dt class='text-muted-foreground'>
              {t('contentVersion.platforms')}
            </dt>
            <dd>
              <ContentVersionBadgeList items={local.version.loaders} />
            </dd>
          </Show>
        </dl>
      </Section>

      <Show when={local.version.files.length > 0}>
        <Section title={t('contentVersion.files')}>
          <ul class='gap-2 flex flex-col'>
            <For each={local.version.files}>
              {(file) => (
                <li class='gap-2 flex items-center justify-between'>
                  <span class='flex min-w-0 flex-col'>
                    <span class='truncate'>{file.fileName}</span>
                    <span class='text-muted-foreground'>
                      {formatBytes(file.size, locale())}
                    </span>
                  </span>
                  <Show when={file.primary}>
                    <Badge variant='secondary'>
                      {t('contentVersion.primaryFile')}
                    </Badge>
                  </Show>
                </li>
              )}
            </For>
          </ul>
        </Section>
      </Show>

      <Show when={local.version.dependencies.length > 0}>
        <Section title={t('contentVersion.dependencies')}>
          <ul class='gap-2 flex flex-col'>
            <For each={local.version.dependencies}>
              {(dependency) => (
                <ContentVersionDependencyItem dependency={dependency} />
              )}
            </For>
          </ul>
        </Section>
      </Show>
    </div>
  );
};
