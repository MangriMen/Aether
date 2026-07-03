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
              <Field of={local.form} path={[name, index(), 0]}>
                {(hostField) => (
                  <Field of={local.form} path={[name, index(), 1]}>
                    {(pluginField) => (
                      <EditableAllowedItem<[string, string], [string, string]>
                        {...getEditableAllowedItemArrayProps(
                          index,
                          setEditingIndex,
                          remove,
                          edit,
                        )}
                        item={AllowedPath}
                        editItem={EditAllowedPath}
                        name={String(index())}
                        value={[hostField.input ?? '', pluginField.input ?? '']}
                        error={
                          [hostField.errors?.[0], pluginField.errors?.[0]] as [
                            string,
                            string,
                          ]
                        }
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
