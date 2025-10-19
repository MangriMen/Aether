import type { Component, ComponentProps } from 'solid-js';

import {
  Field,
  getValues,
  validate,
  type FormStore,
} from '@modular-forms/solid';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

import {
  getEditableAllowedItemArrayProps,
  useCustomItemsEditing,
} from '../lib';
import { PluginSettingsSchema, type PluginSettingsSchemaInput } from '../model';
import { AddNewItem } from './AddNewItem';
import { AllowedHost } from './AllowedHost';
import { EditableAllowedItem } from './EditableAllowedItem';
import { EditAllowedHost } from './EditAllowedHost';
import { FieldArrayFor } from './FieldArrayFor';

const name = 'allowedHosts' as const;

export type AllowedHostsCustomItemsProps = ComponentProps<'div'> & {
  form: FormStore<PluginSettingsSchemaInput>;
  onChangePartial?: (values: Partial<PluginSettingsSchemaInput>) => void;
};

export const AllowedHostsCustomItems: Component<
  AllowedHostsCustomItemsProps
> = (props) => {
  const [local, others] = splitProps(props, [
    'form',
    'onChangePartial',
    'class',
  ]);

  const handleChange = async () => {
    if (!(await validate(local.form, name))) {
      return;
    }

    const values = getValues(local.form, { shouldValid: true });

    const fieldArray = values[name];

    const schema = PluginSettingsSchema;

    const parsed = schema
      .pick({ [name]: true })
      .safeParse({ [name]: fieldArray });
    if (!parsed.success) return;
    const finalValue = parsed.data[name];

    local.onChangePartial?.({
      allowedHosts: finalValue,
    });
  };

  const {
    editingIndex,
    setEditingIndex,
    startAdding,
    endAdding,
    isAdding,
    add,
    edit,
    remove,
  } = useCustomItemsEditing(() => local.form, name, handleChange);

  return (
    <div class={cn('flex flex-col gap-2', local.class)} {...others}>
      <ul class='flex flex-col rounded-sm bg-black/20'>
        <FieldArrayFor of={local.form} name={name}>
          {({ index }) => (
            <li>
              <Field of={local.form} name={`${name}.${index()}`}>
                {(field) => (
                  <EditableAllowedItem
                    {...getEditableAllowedItemArrayProps(
                      index,
                      setEditingIndex,
                      remove,
                      edit,
                    )}
                    item={AllowedHost}
                    editItem={EditAllowedHost}
                    name={field.name}
                    value={field.value}
                    error={field.error}
                    editing={editingIndex() === index()}
                  />
                )}
              </Field>
            </li>
          )}
        </FieldArrayFor>
      </ul>

      <AddNewItem
        isAdding={isAdding()}
        onAddingStart={startAdding}
        onAdd={add}
        onCancel={endAdding}
        children={({ onAdd, onCancel }) => (
          <EditAllowedHost onSave={onAdd} onCancel={onCancel} />
        )}
      />
    </div>
  );
};
