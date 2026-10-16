import type { Component, ComponentProps } from 'solid-js';

import { Field, getErrors, validate, type FormStore } from '@formisch/solid';
import { splitProps } from 'solid-js';
import * as v from 'valibot';

import { cn } from '@/shared/lib';

import {
  PluginSettingsSchema,
  type AllowedPathErrors,
  type AllowedPathValue,
  type PluginSettingsSchemaOutput,
} from '../definitions';
import {
  getEditableAllowedItemArrayProps,
  useCustomItemsEditing,
} from '../helpers';
import { AddNewItem } from './AddNewItem';
import { AllowedPath } from './AllowedPath';
import { EditableAllowedItem } from './EditableAllowedItem';
import { EditAllowedPath } from './EditAllowedPath';
import { FieldArrayFor } from './FieldArrayFor';

const name = 'allowedPaths';

export type AllowedPathsCustomItemsProps = ComponentProps<'div'> & {
  form: FormStore<typeof PluginSettingsSchema>;
  onChangePartial?: (values: Partial<PluginSettingsSchemaOutput>) => void;
};

export const AllowedPathsCustomItems: Component<
  AllowedPathsCustomItemsProps
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
                  <Field of={local.form} path={[name, index(), 'hostPath']}>
                    {(hostField) => (
                      <EditableAllowedItem<AllowedPathValue, AllowedPathErrors>
                        {...getEditableAllowedItemArrayProps(
                          index,
                          setEditingIndex,
                          remove,
                          edit,
                        )}
                        item={AllowedPath}
                        editItem={EditAllowedPath}
                        name={String(index())}
                        value={{
                          hostPath: field.input?.hostPath ?? '',
                          pluginPath: field.input?.pluginPath ?? '',
                          readOnly: field.input?.readOnly ?? true,
                        }}
                        error={{
                          hostPath: hostField.errors?.[0],
                          pluginPath: getErrors(local.form, {
                            path: [name, index(), 'pluginPath'],
                          })?.[0],
                        }}
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
      >
        {({ onAdd, onCancel }) => (
          <EditAllowedPath onSave={onAdd} onCancel={onCancel} />
        )}
      </AddNewItem>
    </div>
  );
};
