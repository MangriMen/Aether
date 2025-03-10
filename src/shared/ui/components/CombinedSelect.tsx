import { createMemo, Show, splitProps } from 'solid-js';
import type { Component, JSX, ValidComponent } from 'solid-js';
import type { SelectRootProps } from './Select';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from './Select';
import type { PartialBy } from '@/shared/model';
import type { CollectionNode, PolymorphicProps } from '@kobalte/core';
import type { SelectValueOptions } from '@kobalte/core/select';

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

  return (
    <Select itemComponent={itemComponent()} {...others}>
      <Show
        when={local.children}
        fallback={
          <SelectTrigger class='gap-1.5 whitespace-nowrap px-2'>
            <SelectValue<Option>>
              {(state) => `${state.selectedOption()}`}
            </SelectValue>
          </SelectTrigger>
        }
      >
        {local.children}
      </Show>
      <SelectContent />
    </Select>
  );
};
