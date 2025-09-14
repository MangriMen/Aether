import type { Accessor, JSX } from 'solid-js';

import { Show } from 'solid-js';

import { useTranslation } from '@/shared/model';
import { Button } from '@/shared/ui';

export type AddNewItemProps<T> = {
  children: (
    onSubmitNew: (value: T) => void,
    onCancel: () => void,
  ) => JSX.Element;
  editingIndex: Accessor<null | number>;
  onAddNew: (value: T) => void;
  setEditingIndex: (value: null | number) => void;
};

export const AddNewSettingsItem = <T,>(props: AddNewItemProps<T>) => {
  const [{ t }] = useTranslation();

  const onAddNew = () => {
    props.setEditingIndex(-1);
  };

  const close = () => {
    props.setEditingIndex(null);
  };

  const onSubmitNew = (value: T) => {
    props.onAddNew(value);
    close();
  };

  return (
    <Show
      fallback={
        <Button
          children={t('plugins.addItem')}
          class='size-max px-2 py-1'
          onClick={onAddNew}
          size='sm'
        />
      }
      when={props.editingIndex() === -1}
    >
      {props.children(onSubmitNew, close)}
    </Show>
  );
};
