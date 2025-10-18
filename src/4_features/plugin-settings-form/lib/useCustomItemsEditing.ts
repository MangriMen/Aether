import type {
  FieldArrayPath,
  FieldArrayPathValue,
  FieldValues,
  FormStore,
} from '@modular-forms/solid';
import type { Accessor } from 'solid-js';

import { insert, remove, replace } from '@modular-forms/solid';
import { createMemo, createSignal } from 'solid-js';

export const useCustomItemsEditing = <
  TFieldValues extends FieldValues,
  TFieldName extends FieldArrayPath<TFieldValues>,
>(
  form: Accessor<FormStore<TFieldValues>>,
  name: TFieldName,
) => {
  const [editingIndex, setEditingIndex] = createSignal<number | null>(null);

  const startAdding = () => setEditingIndex(-1);
  const endAdding = () => setEditingIndex(null);

  const isAdding = createMemo(() => editingIndex() === -1);

  const handleAdd = (
    value: FieldArrayPathValue<TFieldValues, TFieldName>[number],
  ) => {
    endAdding();
    insert(form(), name, { value });
  };

  const handleEdit = (
    index: number,
    value: FieldArrayPathValue<TFieldValues, TFieldName>[number],
  ) => {
    replace(form(), name, {
      at: index,
      value,
    });
    setEditingIndex(null);
  };

  const handleRemove = (index: number) => {
    remove(form(), name, {
      at: index,
    });
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
