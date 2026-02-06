import type { PolymorphicProps } from '@kobalte/core';
import type { ValidComponent } from 'solid-js';

import { splitProps } from 'solid-js';

import type { SelectRootProps } from '@/shared/ui';

import { useTranslation } from '@/shared/model';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui';

import { DEFAULT_PER_PAGE_OPTIONS } from '../model/constants';

export type ItemsPerPageSelectProps<
  Option extends number,
  OptGroup extends never,
  T extends ValidComponent = 'div',
> = {
  options?: Array<Option | OptGroup>;
  onChange?: (value: Option) => void;
} & Omit<
  SelectRootProps<Option, OptGroup, T>,
  'options' | 'multiple' | 'onChange'
>;

export const ItemsPerPageSelect = <
  Option extends number,
  OptGroup extends never,
  T extends ValidComponent = 'div',
>(
  props: PolymorphicProps<T, ItemsPerPageSelectProps<Option, OptGroup, T>>,
) => {
  const [local, others] = splitProps(props, ['options']);

  const [{ t }] = useTranslation();

  return (
    <Select
      multiple={false}
      disallowEmptySelection
      options={local.options ?? DEFAULT_PER_PAGE_OPTIONS}
      itemComponent={(props) => (
        <SelectItem item={props.item}>{props.item.textValue}</SelectItem>
      )}
      {...others}
    >
      <SelectTrigger class='gap-1.5 whitespace-nowrap'>
        <SelectValue<number>>
          {(state) => `${t('common.perPage')}: ${state.selectedOption()}`}
        </SelectValue>
      </SelectTrigger>
      <SelectContent />
    </Select>
  );
};
