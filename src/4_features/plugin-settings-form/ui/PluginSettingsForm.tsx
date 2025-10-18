import type { SubmitHandler } from '@modular-forms/solid';
import type { Accessor } from 'solid-js';

import { splitProps, type Component } from 'solid-js';

import { type PluginManifest } from '@/entities/plugins';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Button } from '@/shared/ui';

import {
  usePluginSettingsForm,
  useResetPluginSettingsFormValues,
} from '../lib';
import { type PluginSettingsSchemaValues } from '../model';
import { AllowedHost } from './AllowedHost';
import { AllowedHostsField } from './AllowedHostsField';
import { AllowedItems } from './AllowedItems';
import { AllowedPath } from './AllowedPath';
import { AllowedPathField } from './AllowedPathField';
import { EditAllowedHost } from './EditAllowedHost';
import { EditAllowedPath } from './EditAllowedPath';

export type PluginSettingsFormProps = {
  initialValues: Accessor<PluginSettingsSchemaValues | undefined>;
  pluginManifest: PluginManifest;
  isLoading?: boolean;
  disabled?: boolean;
  onSubmit?: SubmitHandler<PluginSettingsSchemaValues>;
  class?: string;
};

export const PluginSettingsForm: Component<PluginSettingsFormProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'initialValues',
    'pluginManifest',
    'isLoading',
    'disabled',
    'onSubmit',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const [form, { Form }] = usePluginSettingsForm();
  useResetPluginSettingsFormValues(form, local.initialValues);

  return (
    <Form
      class={cn('flex flex-col gap-4', local.class)}
      onSubmit={local.onSubmit}
      {...others}
    >
      <fieldset
        class={cn('flex flex-col gap-2', {
          'text-muted-foreground': local.disabled,
        })}
        disabled={local.disabled}
      >
        <Button class='w-max' type='submit' size='sm' loading={local.isLoading}>
          {t('plugins.saveSettings')}
        </Button>
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
      </fieldset>
    </Form>
  );
};
