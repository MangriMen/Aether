import { Show, type Component } from 'solid-js';

import type { ProviderPluginPreviewDto } from '@/entities/plugins';

import {
  PluginCapabilitiesSection,
  PluginInfoHeader,
} from '@/entities/plugins';
import { useTranslation } from '@/shared/model';
import {
  Button,
  CombinedSelect,
  FieldLabel,
  Image,
  Separator,
} from '@/shared/ui';

export interface ReleaseOption {
  value: string;
  label: string;
}

export type PluginPreviewCardProps = {
  preview: ProviderPluginPreviewDto;
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

  const manifest = () => props.preview.manifest;
  const capabilities = () => props.preview.capabilities;
  const hasReleases = () => props.releaseOptions.length > 0;
  const hasTag = () => props.selectedTag.length > 0;

  return (
    <div class='gap-4 rounded-lg p-4 flex flex-col border'>
      {/* Header: icon + info row */}
      <div class='gap-4 flex items-center'>
        <Image class='h-31 w-max' />
        <PluginInfoHeader
          class='flex-1'
          name={manifest()?.name ?? props.preview.repo}
          version={manifest()?.version ?? ''}
          apiVersion={manifest()?.api_version ?? null}
          authors={props.authors}
          description={manifest()?.description ?? null}
        />
      </div>

      <Separator />

      {/* Capabilities */}
      <Show when={capabilities()}>
        {(c) => <PluginCapabilitiesSection capabilities={c()} />}
      </Show>

      {/* License */}
      <Show when={manifest()?.license}>
        {(license) => (
          <span class='text-sm text-muted-foreground'>
            {t('plugins.pluginLicense')}: {license()}
          </span>
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
          disabled={!hasTag() || props.isInstalling}
        >
          {props.isInstalling ? t('common.installing') : t('plugins.install')}
        </Button>
      </div>
    </div>
  );
};
