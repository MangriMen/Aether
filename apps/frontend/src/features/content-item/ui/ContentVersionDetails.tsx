import {
  For,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
  type JSX,
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

/** A label over its value, so that a long value has the whole width of the card. */
const Field: Component<{ label?: string; children: JSX.Element }> = (props) => (
  <div class='gap-1 flex flex-col'>
    <dt class='text-sm text-muted-foreground'>{props.label}</dt>
    <dd class='min-w-0'>{props.children}</dd>
  </div>
);

/** What the provider says of the version besides its changelog: for the side of the page. */
export const ContentVersionDetails: Component<ContentVersionDetailsProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['version', 'class']);

  const [{ t, locale }] = useTranslation();

  return (
    <div class={cn('gap-4 flex flex-col', local.class)} {...others}>
      <Section title={t('contentVersion.details')}>
        <dl class='gap-3 flex flex-col'>
          <Field label={t('contentVersion.published')}>
            {formatTime(dayjs(local.version.datePublished))}
          </Field>

          <Show when={local.version.gameVersions.length > 0}>
            <Field label={t('contentVersion.gameVersion')}>
              <ContentVersionBadgeList items={local.version.gameVersions} />
            </Field>
          </Show>

          <Show when={local.version.loaders.length > 0}>
            <Field label={t('contentVersion.platforms')}>
              <ContentVersionBadgeList items={local.version.loaders} />
            </Field>
          </Show>
        </dl>
      </Section>

      <Show when={local.version.files.length > 0}>
        <Section title={t('contentVersion.files')}>
          <ul class='gap-2 flex flex-col'>
            <For each={local.version.files}>
              {(file) => (
                <li class='gap-1 flex flex-col'>
                  <span class='break-all'>{file.fileName}</span>
                  <span class='gap-2 text-muted-foreground flex items-center'>
                    {formatBytes(file.size, locale())}
                    <Show when={file.primary}>
                      <Badge variant='secondary'>
                        {t('contentVersion.primaryFile')}
                      </Badge>
                    </Show>
                  </span>
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
