import type { PolymorphicProps } from '@kobalte/core';

import { Show, splitProps } from 'solid-js';

import type { Option } from '../../../shared/model';
import type { CombinedSelectProps } from '../../../shared/ui';
import type { ContentProviderEntry } from '../model';

import { cn } from '../../../shared/lib';
import { useTranslation } from '../../../shared/model';
import { CombinedSelect, Skeleton } from '../../../shared/ui';

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
    <div class={cn('flex items-center gap-2', local.class)} {...others}>
      <span class='text-muted-foreground'>{t('content.provider')}:</span>
      <Show
        when={!local.isLoading}
        fallback={
          <div class='flex size-full h-10 w-32'>
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
