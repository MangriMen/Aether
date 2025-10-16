import type { FormStore } from '@modular-forms/solid';
import type { Accessor, Component, ComponentProps } from 'solid-js';

import { FieldArray, insert, remove, replace } from '@modular-forms/solid';
import { createSignal, For, Show, splitProps } from 'solid-js';

import { useTranslation } from '@/shared/model';
import { LabeledField } from '@/shared/ui';

import type { PluginSettingsSchemaValues } from '../model';
import type { AddNewItemProps } from './AddNewSettingsItem';

import { AddNewSettingsItem } from './AddNewSettingsItem';

export type AllowedItemFieldProps<T> = {
  form: FormStore<PluginSettingsSchemaValues>;
  name: keyof PluginSettingsSchemaValues;
  index: Accessor<number>;
  editing: boolean;
  onEdit: (index: number) => void;
  onRemove: (index: number) => void;
  onEdited: (index: number, value: T) => void;
  onCancelEdit: (index: null) => void;
};

export type AllowedItemProps<T> = {
  value?: T;
  onEdit?: () => void;
  onRemove?: () => void;
  changeable?: boolean;
};

export type AllowedItemsFieldArrayProps<
  TName extends
    keyof PluginSettingsSchemaValues = keyof PluginSettingsSchemaValues,
  TValue extends
    PluginSettingsSchemaValues[TName][number] = PluginSettingsSchemaValues[TName][number],
> = ComponentProps<'div'> & {
  name: TName;
  form: FormStore<PluginSettingsSchemaValues>;
  allowedItem: Component<AllowedItemProps<TValue>>;
  allowedItemField: Component<AllowedItemFieldProps<TValue>>;
  addNewItem: AddNewItemProps<TValue>['children'];
  unchangeableItems?: TValue[];
  label?: string;
};

export const AllowedItems = <
  TName extends
    keyof PluginSettingsSchemaValues = keyof PluginSettingsSchemaValues,
  TValue extends
    PluginSettingsSchemaValues[TName][number] = PluginSettingsSchemaValues[TName][number],
>(
  props: AllowedItemsFieldArrayProps<TName, TValue>,
) => {
  const [local, others] = splitProps(props, [
    'unchangeableItems',
    'form',
    'name',
    'label',
    'allowedItem',
    'allowedItemField',
    'addNewItem',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const [editingIndex, setEditingIndex] = createSignal<number | null>(null);

  const handleAddNew = (value: TValue) =>
    insert(local.form, local.name, { value });

  const handleEdited = (index: number, value: TValue) => {
    replace(local.form, local.name, {
      at: index,
      value,
    });
    setEditingIndex(null);
  };

  const handleRemove = (index: number) => {
    remove(local.form, local.name, {
      at: index,
    });
  };

  return (
    <LabeledField
      class='px-1'
      label={<span class='text-base font-medium'>{local.label}</span>}
      {...others}
    >
      <Show when={local.unchangeableItems?.length}>
        <LabeledField class='px-1' label={t('plugins.fromPlugin')}>
          <ol class='flex flex-col rounded-lg bg-black/20'>
            <For each={local.unchangeableItems}>
              {(item) => (
                <li>
                  <local.allowedItem value={item} changeable={false} />
                </li>
              )}
            </For>
          </ol>
        </LabeledField>
      </Show>
      <LabeledField class='px-1' label={t('plugins.custom')}>
        <ol class='flex flex-col rounded-lg bg-black/20'>
          <FieldArray of={local.form} name={local.name}>
            {(fieldArray) => (
              <For each={fieldArray.items}>
                {(_, index) => (
                  <li>
                    <local.allowedItemField
                      name={local.name}
                      form={local.form}
                      index={index}
                      editing={editingIndex() === index()}
                      onEdit={setEditingIndex}
                      onRemove={handleRemove}
                      onEdited={handleEdited}
                      onCancelEdit={setEditingIndex}
                    />
                  </li>
                )}
              </For>
            )}
          </FieldArray>
        </ol>
        <AddNewSettingsItem
          editingIndex={editingIndex}
          setEditingIndex={setEditingIndex}
          onAddNew={handleAddNew}
          children={local.addNewItem}
        />
      </LabeledField>
    </LabeledField>
  );
};
