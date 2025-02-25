import type { PluginMetadata } from '@/entities/plugins';
import { cn } from '@/shared/lib';
import { Button } from '@/shared/ui';
import type { SubmitHandler } from '@modular-forms/solid';
import { createForm, zodForm } from '@modular-forms/solid';
import { splitProps, type Component } from 'solid-js';
import type { PluginSettingsSchemaValues } from '../../model';
import { PluginSettingsSchema } from '../../model';

import { AllowedItems } from './AllowedItems';
import { AllowedHostsField } from './AllowedHostsField';
import { EditAllowedHost } from './EditAllowedHost';
import { EditAllowedPath } from './EditAllowedPath';
import { AllowedPathField } from './AllowedPathField';
import { AllowedHost } from './AllowedHost';
import { AllowedPath } from './AllowedPath';

export type PluginSettingsFormProps = {
  plugin: PluginMetadata;
  class?: string;
};

export const PluginSettingsForm: Component<PluginSettingsFormProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['plugin', 'class']);

  const [form, { Form }] = createForm<PluginSettingsSchemaValues>({
    validate: zodForm(PluginSettingsSchema),
    initialValues: {
      allowedHosts: ['test'],
      allowedPaths: [],
    },
  });

  const handleSubmit: SubmitHandler<PluginSettingsSchemaValues> = (values) => {
    console.log(values);
  };

  return (
    <Form
      class={cn('flex flex-col gap-4', local.class)}
      onSubmit={handleSubmit}
      {...others}
    >
      <AllowedItems
        label='Allowed hosts'
        name='allowedHosts'
        form={form}
        unchangeableItems={local.plugin.wasm.allowed_hosts}
        allowedItem={AllowedHost}
        allowedItemField={AllowedHostsField}
        addNewItem={(onSubmitNew, onCancel) => (
          <EditAllowedHost onOk={onSubmitNew} onCancel={onCancel} />
        )}
      />
      <AllowedItems
        label='Allowed paths'
        name='allowedPaths'
        form={form}
        unchangeableItems={local.plugin.wasm.allowed_paths}
        allowedItem={AllowedPath}
        allowedItemField={AllowedPathField}
        addNewItem={(onSubmitNew, onCancel) => (
          <EditAllowedPath onOk={onSubmitNew} onCancel={onCancel} />
        )}
      />
      <Button class='w-max' type='submit'>
        Update settings
      </Button>
    </Form>
  );
};
