import { Show, type Component } from 'solid-js';

import {
  PluginAllowedHostsSection,
  PluginCapabilitiesSection,
  PluginHostFunctionsSection,
  PluginInfoHeader,
} from '@/features/plugins';
import { useTranslation } from '@/shared/i18n';
import {
  Button,
  CombinedSelect,
  FieldLabel,
  Image,
  Separator,
} from '@/shared/ui-kit';

import type { ProviderPluginPreview } from '../model/pluginProvider';

export interface ReleaseOption {
  value: string;
  label: string;
}

export type PluginPreviewCardProps = {
  /** The preview of the selected tag; undefined while it loads or when it failed. */
  preview: ProviderPluginPreview | undefined;
  isPreviewLoading: boolean;
  releaseOptions: ReleaseOption[];
  selectedTag: string;
  onReleaseChange: (option: ReleaseOption | null) => void;
  onInstall: () => void;
  onClear: () => void;
  authors: string | undefined;
  isInstalling: boolean;
};

export const PluginPreviewCard: Component<PluginPreviewCardProps> = (props) => {
  const [{ t }] = useTranslation();

  const hasReleases = () => props.releaseOptions.length > 0;
  const canInstall = () => props.preview !== undefined;

  return (
    <div class='gap-4 rounded-lg p-4 flex flex-col border'>
      <Show
        when={props.preview}
        fallback={
          <Show when={props.isPreviewLoading}>
            <p class='text-sm text-muted-foreground'>{t('common.loading')}</p>
          </Show>
        }
      >
        {(preview) => (
          <>
            {/* Header: icon + info row */}
            <div class='gap-4 flex items-center'>
              <Image class='h-31 w-max' />
              <PluginInfoHeader
                class='flex-1'
                name={preview().manifest.name || preview().repo}
                version={preview().manifest.version}
                apiVersion={preview().manifest.api_version ?? null}
                authors={props.authors}
                description={preview().manifest.description ?? null}
              />
            </div>

            <Separator />

            {/* Capabilities */}
            <Show when={preview().capabilities}>
              {(c) => <PluginCapabilitiesSection capabilities={c()} />}
            </Show>

            {/* What the plugin asks for: shown before it is installed */}
            <PluginAllowedHostsSection
              allowedHosts={preview().manifest.allowed_hosts}
            />
            <PluginHostFunctionsSection
              hostFunctions={preview().manifest.host_functions}
            />

            {/* License */}
            <Show when={preview().manifest.license}>
              {(license) => (
                <span class='text-sm text-muted-foreground'>
                  {t('plugins.pluginLicense')}: {license()}
                </span>
              )}
            </Show>
          </>
        )}
      </Show>

      {/* Version selector */}
      <Show when={hasReleases()}>
        <div class='gap-2 flex flex-col'>
          <FieldLabel>{t('plugins.selectVersion')}</FieldLabel>
          <CombinedSelect
            options={props.releaseOptions}
            optionValue='value'
            optionTextValue='label'
            value={
              props.releaseOptions.find((r) => r.value === props.selectedTag) ??
              null
            }
            onChange={props.onReleaseChange}
          />
        </div>
      </Show>

      {/* Actions row */}
      <div class='gap-2 flex justify-end'>
        <Button variant='secondary' onClick={props.onClear}>
          {t('common.clear')}
        </Button>
        <Button
          variant='default'
          onClick={props.onInstall}
          disabled={!canInstall() || props.isInstalling}
        >
          {props.isInstalling ? t('common.installing') : t('plugins.install')}
        </Button>
      </div>
    </div>
  );
};
