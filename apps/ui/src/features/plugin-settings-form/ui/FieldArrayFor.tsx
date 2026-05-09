import type {
  FieldArrayPath,
  FieldArrayStore,
  FieldValues,
  FormStore,
  ResponseData,
} from '@modular-forms/solid';
import type { Accessor, JSX } from 'solid-js';

import { FieldArray } from '@modular-forms/solid';
import { For } from 'solid-js';

export interface FieldArrayForProps<
  TFieldValues extends FieldValues,
  TResponseData extends ResponseData,
  TFieldArrayName extends FieldArrayPath<TFieldValues>,
> {
  of: FormStore<TFieldValues, TResponseData>;
  name: TFieldArrayName;
  children: (props: {
    fieldArray: FieldArrayStore<TFieldValues, TFieldArrayName>;
    index: Accessor<number>;
  }) => JSX.Element;
}

export const FieldArrayFor = <
  TFieldValues extends FieldValues,
  TResponseData extends ResponseData,
  TFieldArrayName extends FieldArrayPath<TFieldValues>,
>(
  props: FieldArrayForProps<TFieldValues, TResponseData, TFieldArrayName>,
) => {
  return (
    <FieldArray of={props.of} name={props.name}>
      {(fieldArray) => (
        <For each={fieldArray.items}>
          {(_, index) => props.children({ fieldArray, index })}
        </For>
      )}
    </FieldArray>
  );
};
