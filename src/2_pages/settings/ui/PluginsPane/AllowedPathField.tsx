import type { Component } from 'solid-js';

import { Field } from '@modular-forms/solid';
import { Show } from 'solid-js';

import type { AllowedItemFieldProps } from './AllowedItems';

import { AllowedPath } from './AllowedPath';
import { EditAllowedPath } from './EditAllowedPath';

export const AllowedPathField: Component<
  AllowedItemFieldProps<[string, string]>
> = (props) => {
  const handleEdit = () => {
    props.onEdit(props.index());
  };

  const handleRemove = () => {
    props.onRemove(props.index());
  };

  const handleEdited = (value: [string, string]) => {
    props.onEdited(props.index(), value);
  };

  const handleCancel = () => {
    props.onCancelEdit(null);
  };

  return (
    <Field name={`${props.name}.${props.index()}`} of={props.form}>
      {(field) => (
        <Show
          fallback={
            <AllowedPath
              onEdit={handleEdit}
              onRemove={handleRemove}
              value={field.value}
            />
          }
          when={props.editing}
        >
          <EditAllowedPath
            onCancel={handleCancel}
            onOk={handleEdited}
            value={field.value}
          />
        </Show>
      )}
    </Field>
  );
};
