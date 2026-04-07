import type { Component, ComponentProps } from 'solid-js';

import { Show, splitProps } from 'solid-js';

import { useImporters } from '@/entities/plugins';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Button, DialogFooter } from '@/shared/ui';
import { ImportInstanceView } from '@/widgets/import-instance';

import type { TabContentProps } from '../model';

export type ImportInstanceProps = TabContentProps & ComponentProps<'div'>;

export const ImportInstance: Component<ImportInstanceProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const pluginsImporters = useImporters();

  const [{ t }] = useTranslation();

  return (
    <div class={cn('flex flex-col', local.class)} {...others}>
      <Show
        when={pluginsImporters.data?.length}
        fallback={
          <div class='flex grow items-center justify-center'>
            <span class='whitespace-pre-line text-center text-lg text-muted-foreground'>
              {t('createInstance.noImportConfigs')}
            </span>
          </div>
        }
      >
        <Show when={pluginsImporters.data}>
          {(importers) => (
            <ImportInstanceView
              importers={importers()}
              footerButtons={
                <DialogFooter class='mt-auto'>
                  <Button type='submit'>{t('createInstance.import')}</Button>
                  <Button
                    variant='secondary'
                    onClick={() => props.onOpenChange?.(false)}
                  >
                    {t('common.cancel')}
                  </Button>
                </DialogFooter>
              }
              onSubmit={() => props.onOpenChange?.(false)}
            />
          )}
        </Show>
      </Show>
    </div>
  );
};
