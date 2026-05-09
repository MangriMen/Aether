import type { ComponentProps, JSX } from 'solid-js';

import { Show, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { LabeledField } from '@/shared/ui';

const labeledFieldClass = 'px-1 text-sm';
const labelClass = 'font-normal';

export interface AllowedItemsProps extends ComponentProps<'div'> {
  label?: string;
  fixedItems?: JSX.Element;
  customItems?: JSX.Element;
}

export const AllowedItems = (props: AllowedItemsProps) => {
  const [local, others] = splitProps(props, [
    'label',
    'fixedItems',
    'customItems',
  ]);

  const [{ t }] = useTranslation();

  return (
    <LabeledField
      label={<span class='text-base font-medium'>{local.label}</span>}
      {...others}
    >
      <Show when={local.fixedItems}>
        {(fixedItems) => (
          <LabeledField
            class={labeledFieldClass}
            label={<span class={labelClass}>{t('plugins.fromPlugin')}</span>}
          >
            {fixedItems()}
          </LabeledField>
        )}
      </Show>

      <LabeledField
        class={cn(labeledFieldClass)}
        label={<span class={labelClass}>{t('plugins.custom')}</span>}
      >
        <Show when={local.customItems}>{local.customItems}</Show>
      </LabeledField>
    </LabeledField>
  );
};
