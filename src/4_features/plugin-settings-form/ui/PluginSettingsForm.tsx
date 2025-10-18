import type { Accessor } from 'solid-js';

import { type SubmitHandler } from '@modular-forms/solid';
import { splitProps, type Component } from 'solid-js';

import type { RuntimeConfig } from '@/entities/plugins';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Button } from '@/shared/ui';

import {
  usePluginSettingsForm,
  useResetPluginSettingsFormValues,
} from '../lib';
import { type PluginSettingsSchemaInput } from '../model';
import { AllowedHost } from './AllowedHost';
import { AllowedHostsCustomItems } from './AllowedHostsCustomItems';
import { AllowedItems } from './AllowedItems';
import { AllowedPath } from './AllowedPath';
import { AllowedPathsCustomItems } from './AllowedPathsCustomItems';
import { FixedItemsList } from './FixedItemsList';

export type PluginSettingsFormProps = {
  initialValues: Accessor<PluginSettingsSchemaInput | undefined>;
  runtimeConfig: RuntimeConfig;
  isLoading?: boolean;
  disabled?: boolean;
  onSubmit?: SubmitHandler<PluginSettingsSchemaInput>;
  onChangePartial?: (values: Partial<PluginSettingsSchemaInput>) => void;
  class?: string;
};

export const PluginSettingsForm: Component<PluginSettingsFormProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'initialValues',
    'runtimeConfig',
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
      class={cn('flex flex-auto flex-col overflow-hidden', local.class)}
      onSubmit={local.onSubmit}
      {...others}
    >
      <fieldset
        class={cn('flex grow basis-0 flex-col gap-2 overflow-y-auto', {
          'text-muted-foreground': local.disabled,
        })}
        disabled={local.disabled}
      >
        <Button class='w-max' type='submit' size='sm' loading={local.isLoading}>
          {t('plugins.saveSettings')}
        </Button>
        <AllowedItems
          label={t('plugins.allowedHosts')}
          fixedItems={
            local.runtimeConfig.allowed_hosts?.length ? (
              <FixedItemsList
                items={local.runtimeConfig.allowed_hosts}
                item={AllowedHost}
              />
            ) : undefined
          }
          customItems={<AllowedHostsCustomItems form={form} />}
        />
        <AllowedItems
          label={t('plugins.allowedPaths')}
          fixedItems={
            local.runtimeConfig.allowed_paths?.length ? (
              <FixedItemsList
                items={local.runtimeConfig.allowed_paths}
                item={AllowedPath}
              />
            ) : undefined
          }
          customItems={<AllowedPathsCustomItems form={form} />}
        />
      </fieldset>
    </Form>
  );
};
