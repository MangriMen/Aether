import type { DialogRootProps } from '@kobalte/core/dialog';
import type { Component } from 'solid-js';

import { createQuery } from '@tanstack/solid-query';
import { Show, splitProps } from 'solid-js';

import { instanceCommands } from '@/entities/instances';
import { useTranslation } from '@/shared/model';
import { Button, DialogFooter } from '@/shared/ui';
import { ImportInstanceView } from '@/widgets/import-instance';

export type ImportInstanceProps = { class?: string } & Pick<
  DialogRootProps,
  'onOpenChange'
>;

export const ImportInstance: Component<ImportInstanceProps> = (props) => {
  const [, others] = splitProps(props, ['class', 'onOpenChange']);

  const [{ t }] = useTranslation();

  const packManagersQuery = createQuery(() => ({
    queryKey: ['pack_managers'],
    queryFn: () => instanceCommands.listPackManagers(),
  }));

  return (
    <div class='flex flex-col grow' {...others}>
      <Show
        when={packManagersQuery.data && packManagersQuery.data.length > 0}
        fallback={
          <div class='flex flex-col items-center justify-center grow'>
            <span class='text-lg text-muted-foreground text-center whitespace-pre-line'>
              {t('createInstance.noImportConfigs')}
            </span>
          </div>
        }
      >
        <ImportInstanceView
          packManagers={packManagersQuery.data!}
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
        />
      </Show>
    </div>
  );
};
