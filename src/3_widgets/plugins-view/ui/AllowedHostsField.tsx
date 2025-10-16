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
    <Field of={props.form} name={`${props.name}.${props.index()}`}>
      {(field) => (
        <Show
          when={props.editing}
          fallback={
            <AllowedHost
              value={field.value}
              onEdit={handleEdit}
              onRemove={handleRemove}
            />
          }
        >
          <EditAllowedHost
            value={field.value}
            onOk={handleEdited}
            onCancel={handleCancel}
          />
        </Show>
      )}
    </Field>
  );
};
