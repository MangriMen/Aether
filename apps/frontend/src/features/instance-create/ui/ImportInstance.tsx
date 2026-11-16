import type { Component, ComponentProps } from 'solid-js';

import { Match, Show, splitProps, Switch } from 'solid-js';

import { ImportInstanceView } from '@/features/instance-import';
import { useImporters } from '@/features/plugins';
import { useTranslation } from '@/shared/i18n';
import { Button, DialogFooter, LoadError, Skeleton } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import type { TabContentProps } from '../model/tabs';

export type ImportInstanceProps = TabContentProps & ComponentProps<'div'>;

export const ImportInstance: Component<ImportInstanceProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const pluginsImporters = useImporters();

  const [{ t }] = useTranslation();

  return (
    <div class={cn('flex flex-col', local.class)} {...others}>
      <Show
        when={pluginsImporters.data?.length ? pluginsImporters.data : undefined}
        fallback={
          <div class='flex grow items-center justify-center'>
            <Switch>
              <Match when={pluginsImporters.isError && !pluginsImporters.data}>
                <LoadError
                  message={t('common.loadFailed')}
                  retryLabel={t('common.retry')}
                  onRetry={() => void pluginsImporters.refetch()}
                />
              </Match>
              <Match when={pluginsImporters.isLoading}>
                <Skeleton class='h-8 w-48' data-testid='importers-loading' />
              </Match>
              <Match when>
                <span
                  class='
                    text-lg text-muted-foreground text-center
                    whitespace-pre-line
                  '
                >
                  {t('createInstance.noImportConfigs')}
                </span>
              </Match>
            </Switch>
          </div>
        }
      >
        {(importers) => (
          <ImportInstanceView
            importers={importers()}
            footerButtons={(isSubmitting) => (
              <DialogFooter>
                <Button type='submit' loading={isSubmitting()}>
                  {t('createInstance.import')}
                </Button>
                <Button
                  variant='secondary'
                  onClick={() => props.onOpenChange?.(false)}
                >
                  {t('common.cancel')}
                </Button>
              </DialogFooter>
            )}
            onSubmit={() => props.onOpenChange?.(false)}
          />
        )}
      </Show>
    </div>
  );
};
