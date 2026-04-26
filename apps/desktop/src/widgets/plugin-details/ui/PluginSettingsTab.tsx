import {
  createMemo,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { useEditPluginSettings, usePluginSettings } from '@/entities/plugins';
import { PluginSettingsForm } from '@/features/plugin-settings-form';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import type { PluginDetailsTabProps } from '../model';

import { usePluginSettingsHandler } from '../lib';

export type PluginSettingsTabProps = ComponentProps<'div'> &
  PluginDetailsTabProps;

export const PluginSettingsTab: Component<PluginSettingsTabProps> = (props) => {
  const [local, others] = splitProps(props, [
    'plugin',
    'isSettingsDisabled',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const pluginSettings = usePluginSettings(
    () => local.plugin.manifest.metadata.id,
  );
  const editPluginSettings = useEditPluginSettings();

  const { initialValues, onChangePartial } = usePluginSettingsHandler({
    pluginId: () => local.plugin.manifest.metadata.id,
    pluginSettings: () => pluginSettings.data,
    editPluginSettings: () => editPluginSettings.mutateAsync,
  });

  const isLoading = createMemo(
    () => pluginSettings.isLoading || editPluginSettings.isPending,
  );

  return (
    <div
      class={cn(
        'flex flex-col',
        {
          'text-muted-foreground': local.isSettingsDisabled,
        },
        local.class,
      )}
      {...others}
    >
      <Show when={local.isSettingsDisabled}>
        <span class='text-xl font-medium leading-10 brightness-125'>
          {t('plugins.disableToChangeSettings')}
        </span>
      </Show>
      <PluginSettingsForm
        class='grow'
        runtimeConfig={local.plugin.manifest.runtime}
        initialValues={initialValues}
        onChangePartial={onChangePartial}
        isLoading={isLoading()}
        disabled={local.isSettingsDisabled}
      />
    </div>
  );
};
