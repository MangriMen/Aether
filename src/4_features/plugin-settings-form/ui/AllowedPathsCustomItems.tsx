import type { Component, ComponentProps } from 'solid-js';

import { Field, type FormStore } from '@modular-forms/solid';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

import type { PluginSettingsSchemaInput } from '../model';

import {
  getEditableAllowedItemArrayProps,
  useCustomItemsEditing,
} from '../lib';
import { AddNewItem } from './AddNewItem';
import { AllowedPath } from './AllowedPath';
import { EditableAllowedItem } from './EditableAllowedItem';
import { EditAllowedPath } from './EditAllowedPath';
import { FieldArrayFor } from './FieldArrayFor';

const name = 'allowedPaths' as const;

export type AllowedPathsCustomItemsProps = ComponentProps<'div'> & {
  form: FormStore<PluginSettingsSchemaInput>;
};

export const AllowedPathsCustomItems: Component<
  AllowedPathsCustomItemsProps
> = (props) => {
  const [local, others] = splitProps(props, ['form', 'class']);

  const {
    editingIndex,
    setEditingIndex,
    startAdding,
    endAdding,
    isAdding,
    add,
    edit,
    remove,
  } = useCustomItemsEditing(() => local.form, name);

  return (
    <div class={cn('flex flex-col gap-2', local.class)} {...others}>
      <ul class='flex flex-col rounded-sm bg-black/20'>
        <FieldArrayFor of={local.form} name={name}>
          {({ index }) => (
            <li>
              <Field of={local.form} name={`${name}.${index()}.0`}>
                {(hostField) => (
                  <Field of={local.form} name={`${name}.${index()}.1`}>
                    {(pluginField) => (
                      <EditableAllowedItem
                        {...getEditableAllowedItemArrayProps(
                          index,
                          setEditingIndex,
                          remove,
                          edit,
                        )}
                        item={AllowedPath}
                        editItem={EditAllowedPath}
                        name={hostField.name}
                        value={[hostField.value ?? '', pluginField.value ?? '']}
                        error={[hostField.error, pluginField.error]}
                        editing={editingIndex() === index()}
                      />
                    )}
                  </Field>
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
          <EditAllowedPath onSave={onAdd} onCancel={onCancel} />
        )}
      />
    </div>
  );
};
