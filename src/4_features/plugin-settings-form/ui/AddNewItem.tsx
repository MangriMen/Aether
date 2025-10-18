import type { Component } from 'solid-js';

import IconMdiPlus from '~icons/mdi/plus';
import { Show } from 'solid-js';

import { useTranslation } from '@/shared/model';
import { CombinedTooltip, IconButton } from '@/shared/ui';

import type { NewAllowedItemProps } from '../model';

export type AddNewItemProps<T> = {
  isAdding?: boolean;
  onAddingStart?: () => void;
  onAdd: (value: T) => void;
  onCancel: () => void;
  children: Component<NewAllowedItemProps<T>>;
};

export const AddNewItem = <T,>(props: AddNewItemProps<T>) => {
  const [{ t }] = useTranslation();

  return (
    <Show
      when={props.isAdding}
      fallback={
        <CombinedTooltip
          class='size-max p-1'
          label={t('plugins.addItem')}
          as={IconButton}
          size='sm'
          icon={IconMdiPlus}
          onClick={props.onAddingStart}
        />
      }
    >
      {props.children({ onAdd: props.onAdd, onCancel: props.onCancel })}
    </Show>
  );
};
