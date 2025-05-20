import { Field } from '@modular-forms/solid';

import { Show } from 'solid-js';
import type { Component } from 'solid-js';
import type { AllowedItemFieldProps } from './AllowedItems';
import { EditAllowedPath } from './EditAllowedPath';
import { AllowedPath } from './AllowedPath';

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
    <Field of={props.form} name={`${props.name}.${props.index()}`}>
      {(field) => (
        <Show
          when={props.editing}
          fallback={
            <AllowedPath
              value={field.value}
              onEdit={handleEdit}
              onRemove={handleRemove}
            />
          }
        >
          <EditAllowedPath
            value={field.value}
            onOk={handleEdited}
            onCancel={handleCancel}
          />
        </Show>
      )}
    </Field>
  );
};
