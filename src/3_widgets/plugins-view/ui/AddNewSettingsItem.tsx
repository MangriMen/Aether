import type { Accessor, JSX } from 'solid-js';

import MdiPlusIcon from '@iconify/icons-mdi/plus';
import { Show } from 'solid-js';

import { useTranslation } from '@/shared/model';
import { CombinedTooltip, IconButton } from '@/shared/ui';

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
      when={props.editingIndex() === -1}
      fallback={
        <CombinedTooltip
          class='size-max p-1'
          label={t('plugins.addItem')}
          as={IconButton}
          size='sm'
          icon={MdiPlusIcon}
          onClick={onAddNew}
        />
      }
    >
      {props.children(onSubmitNew, close)}
    </Show>
  );
};
