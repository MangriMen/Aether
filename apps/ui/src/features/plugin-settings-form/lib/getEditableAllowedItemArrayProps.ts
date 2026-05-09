import type { Accessor } from 'solid-js';

export const getEditableAllowedItemArrayProps = <TValue>(
  index: Accessor<number>,
  setEditingIndex: (index: number | null) => void,
  handleRemove: (index: number) => void,
  handleEdit: (index: number, value: TValue) => void,
) => {
  const onEdit = () => {
    setEditingIndex(index());
  };

  const onRemove = () => {
    handleRemove(index());
  };

  const onSave = (value: TValue) => {
    handleEdit(index(), value);
  };

  const onCancel = () => {
    setEditingIndex(null);
  };

  return { onEdit, onRemove, onSave, onCancel };
};
