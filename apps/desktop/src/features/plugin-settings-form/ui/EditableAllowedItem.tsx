import { createMemo, mergeProps, Show, splitProps } from 'solid-js';

import type { EditableAllowedItemProps } from '../model';

import { AllowedItemActionButtons } from './AllowedItemActionButtons';

export const EditableAllowedItem = <T, E>(
  props: EditableAllowedItemProps<T, E>,
) => {
  const [_local, sharedProps, editItemProps] = splitProps(
    props,
    ['item', 'editItem', 'editable', 'editing', 'onEdit', 'onRemove'],
    ['value', 'error'],
    ['name', 'onSave', 'onCancel'],
  );

  const local = mergeProps(_local, { editable: true });

  const leadingItems = createMemo(() =>
    local.editable ? (
      <AllowedItemActionButtons
        onEdit={local.onEdit}
        onRemove={local.onRemove}
      />
    ) : undefined,
  );

  return (
    <Show
      when={local.editing}
      fallback={<local.item leadingItems={leadingItems()} {...sharedProps} />}
    >
      <local.editItem {...editItemProps} {...sharedProps} />
    </Show>
  );
};
