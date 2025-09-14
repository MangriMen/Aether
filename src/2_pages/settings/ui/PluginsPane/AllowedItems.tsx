import type { FormStore } from '@modular-forms/solid';
import type { Accessor, Component, ComponentProps } from 'solid-js';

import { FieldArray, insert, remove, replace } from '@modular-forms/solid';
import { createSignal, For, Show, splitProps } from 'solid-js';

import { useTranslation } from '@/shared/model';
import { LabeledField } from '@/shared/ui';

import type { PluginSettingsSchemaValues } from '../../model';
import type { AddNewItemProps } from './AddNewSettingsItem';

import { AddNewSettingsItem } from './AddNewSettingsItem';

export type AllowedItemFieldProps<T> = {
  editing: boolean;
  form: FormStore<PluginSettingsSchemaValues>;
  index: Accessor<number>;
  name: keyof PluginSettingsSchemaValues;
  onCancelEdit: (index: null) => void;
  onEdit: (index: number) => void;
  onEdited: (index: number, value: T) => void;
  onRemove: (index: number) => void;
};

export type AllowedItemProps<T> = {
  changeable?: boolean;
  onEdit?: () => void;
  onRemove?: () => void;
  value?: T;
};

export type AllowedItemsFieldArrayProps<
  TName extends
    keyof PluginSettingsSchemaValues = keyof PluginSettingsSchemaValues,
  TValue extends
    PluginSettingsSchemaValues[TName][number] = PluginSettingsSchemaValues[TName][number],
> = {
  addNewItem: AddNewItemProps<TValue>['children'];
  allowedItem: Component<AllowedItemProps<TValue>>;
  allowedItemField: Component<AllowedItemFieldProps<TValue>>;
  form: FormStore<PluginSettingsSchemaValues>;
  label?: string;
  name: TName;
  unchangeableItems?: TValue[];
} & ComponentProps<'div'>;

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

  const [editingIndex, setEditingIndex] = createSignal<null | number>(null);

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
                  <local.allowedItem changeable={false} value={item} />
                </li>
              )}
            </For>
          </ol>
        </LabeledField>
      </Show>
      <LabeledField class='px-1' label={t('plugins.custom')}>
        <ol class='flex flex-col rounded-lg bg-black/20'>
          <FieldArray name={local.name} of={local.form}>
            {(fieldArray) => (
              <For each={fieldArray.items}>
                {(_, index) => (
                  <li>
                    <local.allowedItemField
                      editing={editingIndex() === index()}
                      form={local.form}
                      index={index}
                      name={local.name}
                      onCancelEdit={setEditingIndex}
                      onEdit={setEditingIndex}
                      onEdited={handleEdited}
                      onRemove={handleRemove}
                    />
                  </li>
                )}
              </For>
            )}
          </FieldArray>
        </ol>
        <AddNewSettingsItem
          children={local.addNewItem}
          editingIndex={editingIndex}
          onAddNew={handleAddNew}
          setEditingIndex={setEditingIndex}
        />
      </LabeledField>
    </LabeledField>
  );
};
