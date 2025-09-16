import type { Component, ComponentProps } from 'solid-js';

import { Show, splitProps } from 'solid-js';

import { useImportConfigs } from '@/entities/instances';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import type { TabContentProps } from '../model';

import { ImportInstanceForm } from './ImportInstanceForm';

export type ImportInstanceProps = TabContentProps & ComponentProps<'div'>;

export const ImportInstance: Component<ImportInstanceProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const importConfigs = useImportConfigs();

  const [{ t }] = useTranslation();

  return (
    <div class={cn('flex flex-col', local.class)} {...others}>
      <Show
        when={importConfigs.data?.length}
        fallback={
          <div class='flex grow items-center justify-center'>
            <span class='whitespace-pre-line text-center text-lg text-muted-foreground'>
              {t('createInstance.noImportConfigs')}
            </span>
          </div>
        }
      >
        <ImportInstanceForm importConfigs={() => importConfigs.data} />
      </Show>
    </div>
  );
};
