import type {
  FormSchema,
  FormStore,
  RequiredPath,
  ValidArrayPath,
} from '@formisch/solid';
import type { JSX } from 'solid-js';
import type { InferInput } from 'valibot';

import { FieldArray } from '@formisch/solid';
import { For } from 'solid-js';

export interface FieldArrayForProps<TSchema extends FormSchema> {
  of: FormStore<TSchema>;
  name: string;
  children: (props: { index: () => number }) => JSX.Element;
}

export const FieldArrayFor = <TSchema extends FormSchema>(
  props: FieldArrayForProps<TSchema>,
) => {
  return (
    <FieldArray
      of={props.of}
      path={
        [props.name] as unknown as ValidArrayPath<
          InferInput<TSchema>,
          [typeof props.name] & RequiredPath
        >
      }
    >
      {(fieldArray) => (
        <For each={fieldArray.items}>
          {(_, getIndex) => props.children({ index: getIndex })}
        </For>
      )}
    </FieldArray>
  );
};
