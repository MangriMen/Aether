import { useTranslate } from '@/shared/model';
import { Button } from '@/shared/ui';
import { Show } from 'solid-js';
import type { Accessor, JSX } from 'solid-js';

export type AddNewItemProps<T> = {
  editingIndex: Accessor<number | null>;
  setEditingIndex: (value: number | null) => void;
  onAddNew: (value: T) => void;
  children: (
    onSubmitNew: (value: T) => void,
    onCancel: () => void,
  ) => JSX.Element;
};

export const AddNewSettingsItem = <T,>(props: AddNewItemProps<T>) => {
  const [{ t }] = useTranslate();

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
      when={props.editingIndex() === -1}
      fallback={
        <Button
          class='size-max px-2 py-1'
          size='sm'
          onClick={onAddNew}
          children={t('plugins.addItem')}
        />
      }
    >
      {props.children(onSubmitNew, close)}
    </Show>
  );
};
