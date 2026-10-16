import type { Accessor } from 'solid-js';

import { Form } from '@formisch/solid';
import { splitProps, type Component } from 'solid-js';

import type { RuntimeConfig } from '@/entities/plugins';

import { cn, noop } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import type { PluginSettingsSchemaInput } from '../definitions';

import {
  usePluginSettingsForm,
  useResetPluginSettingsFormValues,
} from '../helpers';
import { AllowedHost } from './AllowedHost';
import { AllowedHostsCustomItems } from './AllowedHostsCustomItems';
import { AllowedItems } from './AllowedItems';
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
        {/* No fixed folders from the manifest: the core does not mount `runtime.allowedPaths`,
            so listing them would show access the plugin does not have. */}
        <AllowedItems
          label={t('plugins.allowedPaths')}
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
