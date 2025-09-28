import type { SubmitHandler } from '@modular-forms/solid';

import { createForm, setValues, zodForm } from '@modular-forms/solid';
import { createEffect, createMemo, splitProps, type Component } from 'solid-js';

import {
  type PluginManifest,
  type PluginSettings,
  usePluginSettings,
  useEditPluginSettings,
} from '@/entities/plugins';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Button, showToast } from '@/shared/ui';

import type { PluginSettingsSchemaValues } from '../../model';

import { PluginSettingsSchema } from '../../model';
import { AllowedHost } from './AllowedHost';
import { AllowedHostsField } from './AllowedHostsField';
import { AllowedItems } from './AllowedItems';
import { AllowedPath } from './AllowedPath';
import { AllowedPathField } from './AllowedPathField';
import { EditAllowedHost } from './EditAllowedHost';
import { EditAllowedPath } from './EditAllowedPath';

export type PluginSettingsFormProps = {
  pluginManifest: PluginManifest;
  disabled: boolean;
  class?: string;
};

export const PluginSettingsForm: Component<PluginSettingsFormProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'pluginManifest',
    'disabled',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const pluginId = createMemo(() => local.pluginManifest.metadata.id);

  const settings = usePluginSettings(() => pluginId());

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
    if (settings.data) {
      resetForm(settings.data);
    }
  });

  const editPluginSettings = useEditPluginSettings();

  const isLoading = createMemo(
    () => settings.isLoading || editPluginSettings.isPending,
  );

  const handleSubmit: SubmitHandler<PluginSettingsSchemaValues> = async (
    values,
  ) => {
    try {
      const newSettings: PluginSettings = {
        allowed_hosts: values.allowedHosts ?? [],
        allowed_paths: values.allowedPaths ?? [],
      };
      await editPluginSettings.mutateAsync({
        id: local.pluginManifest.metadata.id,
        settings: newSettings,
      });
      await settings.refetch();
    } catch {
      showToast({
        title: `Failed to update "${local.pluginManifest.metadata.name}" plugin settings`,
        variant: 'destructive',
      });
    }
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
          label={t('plugins.allowedHosts')}
          name='allowedHosts'
          form={form}
          unchangeableItems={local.pluginManifest.runtime.allowed_hosts}
          allowedItem={AllowedHost}
          allowedItemField={AllowedHostsField}
          addNewItem={(onSubmitNew, onCancel) => (
            <EditAllowedHost onOk={onSubmitNew} onCancel={onCancel} />
          )}
        />
        <AllowedItems
          label={t('plugins.allowedPaths')}
          name='allowedPaths'
          form={form}
          unchangeableItems={local.pluginManifest.runtime.allowed_paths}
          allowedItem={AllowedPath}
          allowedItemField={AllowedPathField}
          addNewItem={(onSubmitNew, onCancel) => (
            <EditAllowedPath onOk={onSubmitNew} onCancel={onCancel} />
          )}
        />
        <Button class='w-max' type='submit' loading={isLoading()}>
          {t('plugins.saveSettings')}
        </Button>
      </fieldset>
    </Form>
  );
};
