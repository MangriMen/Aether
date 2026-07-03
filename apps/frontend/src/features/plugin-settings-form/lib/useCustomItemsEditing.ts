import type {
  DeepPartial,
  FormSchema,
  FormStore,
  PathValue,
  ValidArrayPath,
} from '@formisch/solid';
import type { Accessor } from 'solid-js';
import type { InferInput } from 'valibot';

import { insert, remove, replace } from '@formisch/solid';
import { createMemo, createSignal } from 'solid-js';

// Filters keys whose values are arrays.
// NonNullable handles v.optional(v.array(...)), where InferInput
// gives Type[] | undefined and undefined doesn't extend readonly unknown[].
type ValidArrayKeys<T> = {
  [K in keyof T]: NonNullable<T[K]> extends readonly unknown[] ? K : never;
}[keyof T] &
  string;

export const useCustomItemsEditing = <
  TSchema extends FormSchema,
  TName extends ValidArrayKeys<InferInput<TSchema>>,
>(
  form: Accessor<FormStore<TSchema>>,
  name: TName,
  onChange?: () => void,
) => {
  type ArrayItem = PathValue<InferInput<TSchema>, readonly [TName, number]>;

  const arrayPath = [name] as unknown as ValidArrayPath<
    InferInput<TSchema>,
    readonly [TName]
  >;

  const [editingIndex, setEditingIndex] = createSignal<number | null>(null);

  const startAdding = () => setEditingIndex(-1);
  const endAdding = () => setEditingIndex(null);

  const isAdding = createMemo(() => editingIndex() === -1);

  const handleAdd = (value: ArrayItem) => {
    endAdding();
    insert(form(), {
      path: arrayPath,
      initialInput: value as DeepPartial<ArrayItem>,
    });
    onChange?.();
  };

  const handleEdit = (index: number, value: ArrayItem) => {
    replace(form(), {
      path: arrayPath,
      at: index,
      initialInput: value as DeepPartial<ArrayItem>,
    });
    setEditingIndex(null);
    onChange?.();
  };

  const handleRemove = (index: number) => {
    remove(form(), {
      path: arrayPath,
      at: index,
    });
    onChange?.();
  };

  return {
    editingIndex,
    setEditingIndex,
    startAdding,
    endAdding,
    isAdding,
    add: handleAdd,
    edit: handleEdit,
    remove: handleRemove,
  };
};
