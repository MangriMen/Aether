import type { Component, ComponentProps } from 'solid-js';

import { Show, splitProps } from 'solid-js';

import { useImporters } from '@/entities/instances';
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
        when={pluginsImporters.data?.length ? pluginsImporters.data : undefined}
        fallback={
          <div class='flex grow items-center justify-center'>
            <span
              class='
                text-lg text-muted-foreground text-center whitespace-pre-line
              '
            >
              {t('createInstance.noImportConfigs')}
            </span>
          </div>
        }
      >
        {(importers) => (
          <ImportInstanceView
            importers={importers()}
            footerButtons={
              <DialogFooter>
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
    </div>
  );
};
