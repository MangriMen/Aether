import { A, useParams } from '@solidjs/router';
import IconMdiArrowLeft from '~icons/mdi/arrow-left';
import IconMdiOpenInNew from '~icons/mdi/open-in-new';
import {
  Match,
  Show,
  splitProps,
  Switch,
  type Component,
  type ComponentProps,
} from 'solid-js';
import { Dynamic } from 'solid-js/web';

import {
  createContentPageHref,
  useContent,
  useContentContext,
  useContentVersion,
} from '@/features/content';
import { isUnsupportedOperationError } from '@/features/errors';
import { useTranslation } from '@/shared/i18n';
import { MarkdownRenderer } from '@/shared/markdown';
import { Button, CombinedTooltip, IconButton, Skeleton } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import { useContentVersionInstall } from '../lib/useContentVersionInstall';
import { ContentVersionDetails } from './ContentVersionDetails';
import { ContentVersionTypeBadge } from './ContentVersionTypeBadge';

export type ContentVersionPageProps = ComponentProps<'div'>;

/** One version of a content item: what the provider says of it, and the way to install it. */
export const ContentVersionPage: Component<ContentVersionPageProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['class']);

  const [{ t }] = useTranslation();
  const [context] = useContentContext();
  const params = useParams();

  const item = useContent(() =>
    params.contentId && context.providerId
      ? { contentId: params.contentId, providerId: context.providerId }
      : undefined,
  );

  const version = useContentVersion(() =>
    params.versionId && context.providerId
      ? { versionId: params.versionId, providerId: context.providerId }
      : undefined,
  );

  const { install, icon, isCurrentVersionInstalled, isSwap, isInstalling } =
    useContentVersionInstall(
      () => version.data,
      () => item.data?.contentType,
    );

  const projectHref = () =>
    createContentPageHref(params.contentId ?? '', {
      instanceId: context.instanceId,
      providerId: context.providerId,
    });

  const installLabel = () => {
    if (isCurrentVersionInstalled()) {
      return t('common.installed');
    }

    return isSwap() ? t('contentVersion.switchToVersion') : t('common.install');
  };

  return (
    <div
      class={cn('gap-4 p-0.5 flex grow flex-col overflow-auto', local.class)}
      {...others}
    >
      <A
        href={projectHref()}
        class='
          gap-1 text-muted-foreground flex w-fit items-center
          hover:text-foreground
        '
      >
        <IconMdiArrowLeft />
        {item.data?.name ?? t('contentVersion.backToProject')}
      </A>

      <Switch>
        <Match
          when={version.isError && isUnsupportedOperationError(version.error)}
        >
          <p class='text-muted-foreground'>{t('contentVersion.unsupported')}</p>
        </Match>
        <Match when={version.isError}>
          <p class='text-destructive'>{t('contentVersion.loadFailed')}</p>
        </Match>
        <Match when={version.data}>
          {(data) => (
            <>
              <div class='gap-3 flex flex-wrap items-center justify-between'>
                <div class='gap-3 flex min-w-0 items-center'>
                  <ContentVersionTypeBadge type={data().versionType} />
                  <div class='flex min-w-0 flex-col'>
                    <h1 class='text-2xl font-bold text-foreground'>
                      {data().name || data().versionNumber}
                    </h1>
                    <span class='text-muted-foreground'>
                      {data().versionNumber}
                    </span>
                  </div>
                </div>

                <div class='gap-2 flex'>
                  <Button
                    onClick={install}
                    loading={isInstalling()}
                    disabled={isCurrentVersionInstalled() || !item.data}
                  >
                    <Dynamic component={icon()} />
                    {installLabel()}
                  </Button>
                  <Show when={data().webUrl}>
                    {(href) => (
                      <CombinedTooltip
                        label={t('content.openInBrowser')}
                        as='div'
                        class='size-min'
                      >
                        <IconButton
                          variant='secondary'
                          as='a'
                          href={href()}
                          target='_blank'
                          rel='noreferrer'
                          icon={IconMdiOpenInNew}
                        />
                      </CombinedTooltip>
                    )}
                  </Show>
                </div>
              </div>

              <div class='gap-4 lg:grid-cols-[1fr_20rem] grid grid-cols-1 items-start'>
                <section class='min-w-0 rounded-md border bg-card/card'>
                  <h2 class='px-4 pt-4 text-base font-semibold'>
                    {t('contentVersion.changelog')}
                  </h2>
                  <Show
                    when={data().changelog?.trim()}
                    fallback={
                      <p class='p-4 text-muted-foreground'>
                        {t('contentVersion.noChangelog')}
                      </p>
                    }
                  >
                    {(changelog) => (
                      <MarkdownRenderer class='p-4' children={changelog()} />
                    )}
                  </Show>
                </section>

                <ContentVersionDetails version={data()} />
              </div>
            </>
          )}
        </Match>
        <Match when>
          <Skeleton height={96} />
        </Match>
      </Switch>
    </div>
  );
};
