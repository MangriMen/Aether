import type { Accessor } from 'solid-js';

import { Form } from '@formisch/solid';
import { splitProps, type Component } from 'solid-js';

import type { RuntimeConfig } from '@/entities/plugins';

import { cn, noop } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import type { PluginSettingsSchemaInput } from '../model';

import {
  usePluginSettingsForm,
  useResetPluginSettingsFormValues,
} from '../lib';
import { AllowedHost } from './AllowedHost';
import { AllowedHostsCustomItems } from './AllowedHostsCustomItems';
import { AllowedItems } from './AllowedItems';
import { AllowedPath } from './AllowedPath';
import { AllowedPathsCustomItems } from './AllowedPathsCustomItems';
import { FixedItemsList } from './FixedItemsList';

export type PluginSettingsFormProps = {
  initialValues: Accessor<PluginSettingsSchemaInput | undefined>;
  runtimeConfig: RuntimeConfig;
  disabled?: boolean;
  onChangePartial?: (values: PluginSettingsSchemaInput) => void;
  class?: string;
};

export const PluginSettingsForm: Component<PluginSettingsFormProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'initialValues',
    'runtimeConfig',
    'disabled',
    'onChangePartial',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const form = usePluginSettingsForm();
  useResetPluginSettingsFormValues(form, local.initialValues);

  return (
    <Form
      of={form}
      onSubmit={noop}
      class={cn('flex flex-1 grow flex-col', local.class)}
      {...others}
    >
      <fieldset
        class={cn('gap-2 flex-1 grow flex-col', {
          'text-muted-foreground': local.disabled,
        })}
        disabled={local.disabled}
      >
        <AllowedItems
          label={t('plugins.allowedHosts')}
          fixedItems={
            local.runtimeConfig.allowedHosts?.length ? (
              <FixedItemsList
                items={local.runtimeConfig.allowedHosts}
                item={AllowedHost}
              />
            ) : undefined
          }
          customItems={
            <AllowedHostsCustomItems
              form={form}
              onChangePartial={local.onChangePartial}
            />
          }
        />
        <AllowedItems
          label={t('plugins.allowedPaths')}
          fixedItems={
            local.runtimeConfig.allowedPaths?.length ? (
              <FixedItemsList
                items={local.runtimeConfig.allowedPaths}
                item={AllowedPath}
              />
            ) : undefined
          }
          customItems={
            <AllowedPathsCustomItems
              form={form}
              onChangePartial={local.onChangePartial}
            />
          }
        />
      </fieldset>
    </Form>
  );
};
