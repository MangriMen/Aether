import type { PolymorphicProps } from '@kobalte/core';

import { Show, splitProps } from 'solid-js';

import type { CombinedSelectProps, Option } from '@/shared/ui-kit';

import { useTranslation } from '@/shared/i18n';
import { CombinedSelect, Skeleton } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import type { ContentProviderEntry } from '../model/contentProviderMapper';

export interface ContentProviderSelectProps {
  isLoading?: boolean;
}

export const ContentProviderSelect = <
  Opt extends Option<ContentProviderEntry> | null,
  OptGroup,
>(
  props: ContentProviderSelectProps &
    PolymorphicProps<'div', CombinedSelectProps<Opt, OptGroup, 'div'>>,
) => {
  const [local, selectProps, others] = splitProps(
    props,
    ['isLoading', 'class'],
    ['options', 'value', 'onChange'],
  );

  const [{ t }] = useTranslation();

  return (
    <div class={cn('gap-2 flex items-center', local.class)} {...others}>
      <span class='text-muted-foreground'>{t('content.provider')}:</span>
      <Show
        when={!local.isLoading}
        fallback={
          <div class='h-9 w-32 flex size-full'>
            <Skeleton radius={6} />
          </div>
        }
      >
        <CombinedSelect
          title={t('content.provider')}
          optionValue='value'
          optionTextValue='name'
          {...selectProps}
        />
      </Show>
    </div>
  );
};
