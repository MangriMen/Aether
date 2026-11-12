import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import { cn } from '@/shared/utils';

import type { AllowedItemProps } from '../model/allowedItem';

import { AllowedItemLeadingItems } from './AllowedItemLeadingItems';
import { AllowedTruncatedText } from './AllowedTruncatedText';

export type AllowedHostProps = ComponentProps<'div'> &
  AllowedItemProps<string, string>;

export const AllowedHost: Component<AllowedHostProps> = (props) => {
  const [local, others] = splitProps(props, [
    'value',
    'error',
    'leadingItems',
    'class',
  ]);
  const [{ t }] = useTranslation();

  return (
    <div class={cn('flex flex-col', local.class)} {...others}>
      <div
        class='
          group gap-2 rounded-sm px-2 py-1
          hover:bg-secondary/hover
          flex items-center justify-between select-none
        '
      >
        <AllowedTruncatedText value={local.value} />
        <AllowedItemLeadingItems items={local.leadingItems} />
      </div>
      <span class='text-destructive'>
        {local.error ? t('validation.hostRequired') : undefined}
      </span>
    </div>
  );
};
