import {
  createEffect,
  createMemo,
  createSignal,
  Show,
  type Component,
} from 'solid-js';

import {
  PluginCapabilitiesSection,
  PluginInfoHeader,
  useInstallPluginFromProvider,
  usePluginPreview,
  type PluginCapabilities,
} from '@/entities/plugins';
import { useTranslation } from '@/shared/model';
import {
  Button,
  CombinedSelect,
  CombinedTextField,
  FieldLabel,
  Image,
  Separator,
} from '@/shared/ui';

export type InstallFromGithubViewProps = {
  onInstalled?: () => void;
};

interface ReleaseOption {
  value: string;
  label: string;
}

const tryParseCapabilities = (
  raw: string | null,
): PluginCapabilities | null => {
  if (!raw) return null;
  try {
    return JSON.parse(raw) as PluginCapabilities;
  } catch {
    return null;
  }
};

export const InstallFromGithubView: Component<InstallFromGithubViewProps> = (
  props,
) => {
  const [{ t }] = useTranslation();

  const [url, setUrl] = createSignal('');
  const [shouldFetch, setShouldFetch] = createSignal(false);
  const [selectedTag, setSelectedTag] = createSignal('');

  const previewQuery = usePluginPreview(() =>
    shouldFetch() ? url().trim() : '',
  );
  const installMutation = useInstallPluginFromProvider();

  // Auto-select first release when preview loads
  createEffect(() => {
    const data = previewQuery.data as
      | { releases: Array<{ tag_name: string }> }
      | undefined;
    if (data && data.releases.length > 0 && !selectedTag()) {
      setSelectedTag(data.releases[0]!.tag_name);
    }
  });

  const previewData = createMemo(() => {
    const result = previewQuery.data as
      | {
          owner: string;
          repo: string;
          manifest: {
            id: string;
            name: string;
            version: string;
            description: string | null;
            authors: string[];
            license: string | null;
            api_version: string | null;
          } | null;
          capabilities: string | null;
          releases: Array<{
            tag_name: string;
            version: string;
            is_prerelease: boolean;
            download_url: string;
          }>;
        }
      | undefined;
    return result ?? null;
  });

  const capabilities = createMemo(() =>
    tryParseCapabilities(previewData()?.capabilities ?? null),
  );

  const releaseOptions = createMemo<ReleaseOption[]>(() => {
    const p = previewData();
    if (!p) return [];
    return p.releases.map((r) => ({
      value: r.tag_name,
      label: r.is_prerelease
        ? `${r.tag_name} (${t('plugins.prerelease')})`
        : r.tag_name,
    }));
  });

  const handleReleaseChange = (option: ReleaseOption | null) => {
    if (option) setSelectedTag(option.value);
  };

  const currentReleaseOption = createMemo(
    () => releaseOptions().find((r) => r.value === selectedTag()) ?? null,
  );

  const handlePreview = () => {
    if (!url().trim()) return;
    setSelectedTag('');
    setShouldFetch(true);
  };

  const handleInstall = async () => {
    const p = previewData();
    const tag = selectedTag();
    if (!p || !tag) return;

    const release = p.releases.find((r) => r.tag_name === tag);
    if (!release) return;

    try {
      await installMutation.mutateAsync({
        identifier: url().trim(),
        downloadUrl: release.download_url,
        tagName: release.tag_name,
        version: release.version,
      });
      props.onInstalled?.();
    } catch {
      // Error handled by mutation
    }
  };

  const handleReset = () => {
    setShouldFetch(false);
    setUrl('');
    setSelectedTag('');
  };

  const authorsStr = () => previewData()?.manifest?.authors?.join(', ');

  return (
    <div class='flex flex-col gap-4'>
      {/* URL input row */}
      <div class='flex items-end gap-2'>
        <CombinedTextField
          class='flex-1'
          label={t('plugins.githubUrl')}
          inputProps={{
            placeholder: 'https://github.com/owner/repo',
          }}
          value={url()}
          onChange={setUrl}
        />
        <Button
          variant='default'
          onClick={handlePreview}
          disabled={previewQuery.isLoading || !url().trim()}
        >
          <Show when={!previewQuery.isLoading} fallback={t('common.loading')}>
            {t('plugins.preview')}
          </Show>
        </Button>
      </div>

      {/* Error message */}
      <Show when={previewQuery.isError}>
        <p class='text-sm text-destructive'>
          {previewQuery.error instanceof Error
            ? previewQuery.error.message
            : String(t('plugins.githubPreviewError') ?? 'Preview failed')}
        </p>
      </Show>

      {/* Preview card */}
      <Show when={previewData()}>
        {(p) => (
          <div class='flex flex-col gap-4 rounded-lg border p-4'>
            {/* Header: icon + info row */}
            <div class='flex items-center gap-4'>
              <Image class='h-[124px] w-max' />
              <PluginInfoHeader
                class='flex-1'
                name={p().manifest?.name ?? p().repo}
                version={p().manifest?.version ?? ''}
                apiVersion={p().manifest?.api_version ?? null}
                authors={authorsStr()}
                description={p().manifest?.description ?? null}
              />
            </div>

            <Separator />

            {/* Capabilities */}
            <PluginCapabilitiesSection capabilities={capabilities()} />

            {/* License */}
            <Show when={p().manifest?.license}>
              <span class='text-sm text-muted-foreground'>
                {t('plugins.pluginLicense')}: {p().manifest!.license}
              </span>
            </Show>

            {/* Version selector */}
            <Show when={releaseOptions().length > 0}>
              <div class='flex flex-col gap-2'>
                <FieldLabel>{t('plugins.selectVersion')}</FieldLabel>
                <CombinedSelect
                  options={releaseOptions()}
                  optionValue='value'
                  optionTextValue='label'
                  value={currentReleaseOption()}
                  onChange={handleReleaseChange}
                />
              </div>
            </Show>

            {/* Actions row */}
            <div class='flex justify-end gap-2'>
              <Button variant='secondary' onClick={handleReset}>
                {t('common.clear')}
              </Button>
              <Button
                variant='default'
                onClick={handleInstall}
                disabled={!selectedTag() || installMutation.isPending}
              >
                {installMutation.isPending
                  ? t('common.installing')
                  : t('plugins.install')}
              </Button>
            </div>
          </div>
        )}
      </Show>
    </div>
  );
};
