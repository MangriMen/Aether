import type { CollectionNode, PolymorphicProps } from '@kobalte/core';
import type { SelectValueOptions } from '@kobalte/core/select';
import type { Component, JSX, ValidComponent } from 'solid-js';

import { Show, splitProps } from 'solid-js';

import type { PartialBy } from '@/shared/utils';

import type { SelectRootProps } from '../primitives/Select';

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../primitives/Select';

export type CombinedSelectProps<
  Option,
  OptGroup,
  T extends ValidComponent = 'div',
> = PartialBy<
  PolymorphicProps<T, SelectRootProps<Option, OptGroup, T>>,
  'itemComponent'
> & {
  selectValueComponent?: JSX.Element | Component<SelectValueOptions<Option>>;
};

type SelectValueState<Option> = Parameters<
  Extract<SelectValueOptions<Option>['children'], (...args: never[]) => unknown>
>[0];

const DefaultItem = <Option,>(props: { item: CollectionNode<Option> }) => (
  <SelectItem item={props.item}>{props.item.textValue}</SelectItem>
);

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

  const getSelectedValue = (state: SelectValueState<Option>): string | null => {
    const selected = state.selectedOption();

    if (selected == null) {
      return null;
    }

    const optionTextValue = props.optionTextValue;

    if (typeof optionTextValue === 'function') {
      return optionTextValue(selected);
    }

    if (
      (typeof optionTextValue === 'string' ||
        typeof optionTextValue === 'number') &&
      typeof selected === 'object'
    ) {
      const text = (selected as Record<PropertyKey, unknown>)[optionTextValue];

      return text == null ? null : String(text);
    }

    if (
      typeof selected === 'string' ||
      typeof selected === 'number' ||
      typeof selected === 'boolean'
    ) {
      return String(selected);
    }

    return null;
  };

  return (
    <Select
      class={local.class}
      itemComponent={local.itemComponent ?? DefaultItem}
      {...others}
    >
      <Show
        when={local.children}
        fallback={
          <SelectTrigger class='gap-1.5 whitespace-nowrap'>
            <SelectValue<Option>>
              {(state) => getSelectedValue(state)}
            </SelectValue>
          </SelectTrigger>
        }
      >
        {(content) => <>{content()}</>}
      </Show>
      <SelectContent />
    </Select>
  );
};
