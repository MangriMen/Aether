import type { Component, ComponentProps } from 'solid-js';

import { Field, validate, type FormStore } from '@formisch/solid';
import { splitProps } from 'solid-js';
import * as v from 'valibot';

import { cn } from '@/shared/lib';

import {
  getEditableAllowedItemArrayProps,
  useCustomItemsEditing,
} from '../lib';
import {
  PluginSettingsSchema,
  type PluginSettingsSchemaOutput,
} from '../model';
import { AddNewItem } from './AddNewItem';
import { AllowedHost } from './AllowedHost';
import { EditableAllowedItem } from './EditableAllowedItem';
import { EditAllowedHost } from './EditAllowedHost';
import { FieldArrayFor } from './FieldArrayFor';

const name = 'allowedHosts';

export type AllowedHostsCustomItemsProps = ComponentProps<'div'> & {
  form: FormStore<typeof PluginSettingsSchema>;
  onChangePartial?: (values: Partial<PluginSettingsSchemaOutput>) => void;
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
    const result = await validate(local.form);

    if (!result.success) {
      return;
    }

    const TargetSchema = v.pick(PluginSettingsSchema, [name]);
    const parsed = v.safeParse(TargetSchema, result.output);

    if (!parsed.success) {
      return;
    }

    const finalValue = parsed.output[name];

    local.onChangePartial?.({
      [name]: finalValue,
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
    <div class={cn('gap-2 flex flex-col', local.class)} {...others}>
      <ul class='rounded-sm bg-black/20 flex flex-col'>
        <FieldArrayFor of={local.form} name={name}>
          {({ index }) => (
            <li>
              <Field of={local.form} path={[name, index()]}>
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
                    name={String(index())}
                    value={field.input}
                    error={field.errors?.[0]}
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
      >
        {({ onAdd, onCancel }) => (
          <EditAllowedHost onSave={onAdd} onCancel={onCancel} />
        )}
      </AddNewItem>
    </div>
  );
};
