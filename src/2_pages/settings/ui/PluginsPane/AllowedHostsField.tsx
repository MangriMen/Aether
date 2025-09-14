import type { Component } from 'solid-js';

import { Field } from '@modular-forms/solid';
import { Show } from 'solid-js';

import type { AllowedItemFieldProps } from './AllowedItems';

import { AllowedHost } from './AllowedHost';
import { EditAllowedHost } from './EditAllowedHost';

export const AllowedHostsField: Component<AllowedItemFieldProps<string>> = (
  props,
) => {
  const handleEdit = () => {
    props.onEdit(props.index());
  };

  const handleRemove = () => {
    props.onRemove(props.index());
  };

  const handleEdited = (value: string) => {
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
            <AllowedHost
              onEdit={handleEdit}
              onRemove={handleRemove}
              value={field.value}
            />
          }
          when={props.editing}
        >
          <EditAllowedHost
            onCancel={handleCancel}
            onOk={handleEdited}
            value={field.value}
          />
        </Show>
      )}
    </Field>
  );
};
