import { editPluginSettings, getPluginSettings } from '@/entities/plugins';
import type { PluginSettings, PluginMetadata } from '@/entities/plugins';

import { cn } from '@/shared/lib';
import { Button, showToast } from '@/shared/ui';
import type { SubmitHandler } from '@modular-forms/solid';
import { createForm, setValues, zodForm } from '@modular-forms/solid';
import {
  createEffect,
  createMemo,
  createResource,
  createSignal,
  splitProps,
  type Component,
} from 'solid-js';
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
  disabled: boolean;
  class?: string;
};

export const PluginSettingsForm: Component<PluginSettingsFormProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['plugin', 'disabled', 'class']);

  const pluginId = createMemo(() => local.plugin.plugin.id);
  const [settings, { mutate }] = createResource(pluginId, getPluginSettings);

  const [isLoading, setIsLoading] = createSignal(false);

  const [form, { Form }] = createForm<PluginSettingsSchemaValues>({
    validate: zodForm(PluginSettingsSchema),
    initialValues: {
      allowedHosts: [],
      allowedPaths: [],
    },
  });

  const resetForm = (settings?: PluginSettings) => {
    if (form.submitting) {
      return;
    }
    setValues(form, {
      allowedHosts: settings?.allowed_hosts ?? [],
      allowedPaths: settings?.allowed_paths ?? [],
    });
  };

  createEffect(() => {
    resetForm(settings());
  });

  const handleSubmit: SubmitHandler<PluginSettingsSchemaValues> = async (
    values,
  ) => {
    const dto: PluginSettings = {
      allowed_hosts: values.allowedHosts ?? [],
      allowed_paths: values.allowedPaths ?? [],
    };
    setIsLoading(true);
    try {
      await editPluginSettings(local.plugin.plugin.id, dto);
      mutate(dto);
    } catch {
      showToast({
        title: `Failed to update "${local.plugin.plugin.name}" plugin settings`,
        variant: 'destructive',
      });
    }
    setIsLoading(false);
  };

  return (
    <Form
      class={cn('flex flex-col gap-4', local.class)}
      onSubmit={handleSubmit}
      {...others}
    >
      <fieldset
        class={cn('flex flex-col gap-4', {
          'text-muted-foreground': local.disabled,
        })}
        disabled={local.disabled}
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
        <Button class='w-max' type='submit' loading={isLoading()}>
          Save settings
        </Button>
      </fieldset>
    </Form>
  );
};
