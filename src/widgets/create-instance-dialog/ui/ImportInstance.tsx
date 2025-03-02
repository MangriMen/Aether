import { createEffect, Show } from 'solid-js';
import { refetchImportConfigs, useImportConfigs } from '@/entities/instances';
import type { DialogRootProps } from '@kobalte/core/dialog';
import type { Component, ComponentProps } from 'solid-js';
import { ImportInstanceForm } from './ImportInstanceForm';
import { useTranslate } from '@/shared/model';

export type ImportInstanceProps = Omit<ComponentProps<'form'>, 'onSubmit'> &
  Pick<DialogRootProps, 'onOpenChange'>;

export const ImportInstance: Component<ImportInstanceProps> = (props) => {
  const importHandlers = useImportConfigs();

  const [{ t }] = useTranslate();

  createEffect(() => {
    refetchImportConfigs();
  });

  return (
    <Show
      when={importHandlers().length}
      fallback={
        <div class='flex h-full items-center justify-center'>
          <span class='whitespace-pre-line text-center text-lg text-muted-foreground'>
            {t('createInstance.noImportConfigs')}
          </span>
        </div>
      }
    >
      <ImportInstanceForm importHandlers={importHandlers} {...props} />
    </Show>
  );
};
