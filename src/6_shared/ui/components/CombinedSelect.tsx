import type { CollectionNode, PolymorphicProps } from '@kobalte/core';
import type { SelectValueOptions } from '@kobalte/core/select';
import type { Component, JSX, ValidComponent } from 'solid-js';

import { createMemo, Show, splitProps } from 'solid-js';

import type { PartialBy } from '@/shared/model';

import type { SelectRootProps } from './Select';

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from './Select';

export type CombinedSelectProps<
  Option,
  OptGroup,
  T extends ValidComponent = 'div',
> = {
  selectValueComponent?: Component<SelectValueOptions<Option>> | JSX.Element;
} & PartialBy<
  PolymorphicProps<T, SelectRootProps<Option, OptGroup, T>>,
  'itemComponent'
>;

export const CombinedSelect = <
  Option,
  OptGroup,
  T extends ValidComponent = 'div',
>(
  props: CombinedSelectProps<Option, OptGroup, T>,
) => {
  const [local, others] = splitProps(props, [
    'itemComponent',
    'class',
    'children',
  ]);

  const itemComponent = createMemo(
    () =>
      local.itemComponent ??
      ((props: { item: CollectionNode<Option> }) => (
        <SelectItem item={props.item}>{props.item.textValue}</SelectItem>
      )),
  );

  const getSelectedValue = (state: {
    clear: () => void;
    remove: (option: Option) => void;
    selectedOption: () => Option;
    selectedOptions: () => Option[];
  }): null | string => {
    if (!state) {
      return null;
    }

    const selectedOption = state.selectedOption();

    if (!selectedOption) {
      return null;
    }

    if (typeof selectedOption === 'string') {
      return selectedOption;
    } else if (typeof selectedOption === 'object') {
      if (typeof props.optionTextValue === 'string') {
        return selectedOption[props.optionTextValue];
      }
    }

    return null;
  };

  return (
    <Select itemComponent={itemComponent()} {...others}>
      <Show
        fallback={
          <SelectTrigger class='gap-1.5 whitespace-nowrap px-2'>
            <SelectValue<Option>>
              {(state) => getSelectedValue(state)}
            </SelectValue>
          </SelectTrigger>
        }
        when={local.children}
      >
        {local.children}
      </Show>
      <SelectContent />
    </Select>
  );
};
