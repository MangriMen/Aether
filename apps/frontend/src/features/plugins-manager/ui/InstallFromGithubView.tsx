import {
  createEffect,
  createMemo,
  createSignal,
  Show,
  type Component,
} from 'solid-js';

import {
  useInstallPluginFromProvider,
  usePluginPreview,
} from '@/features/plugins';
import { useTranslation } from '@/shared/i18n';
import { CombinedTextField } from '@/shared/ui-kit';
import { debounce } from '@/shared/utils';

import type { ProviderPluginPreview } from '../model/pluginProvider';
import type { ReleaseOption } from './PluginPreviewCard';

import { resolveReleaseTag } from '../model/resolveReleaseTag';
import { PluginPreviewCard } from './PluginPreviewCard';

export type InstallFromGithubViewProps = {
  onInstalled?: () => void;
};

const DEBOUNCE_MS = 500;

export const InstallFromGithubView: Component<InstallFromGithubViewProps> = (
  props,
) => {
  const [{ t }] = useTranslation();

  const [url, setUrl] = createSignal('');
  const [debouncedUrl, setDebouncedUrl] = createSignal('');

  const debounceUrl = debounce(
    (value: string) => setDebouncedUrl(value),
    DEBOUNCE_MS,
  );

  const handleUrlChange = (value: string) => {
    setUrl(value);
    debounceUrl(value);
  };

  const handleUrlClear = () => {
    setUrl('');
    setDebouncedUrl('');
  };

  const previewQuery = usePluginPreview(debouncedUrl);
  const installMutation = useInstallPluginFromProvider();

  const previewData = createMemo<ProviderPluginPreview | null | undefined>(
    () => previewQuery.data,
  );

  const [pickedTag, setPickedTag] = createSignal('');

  // A pick belongs to the repository it was made in; a new URL or release list invalidates it.
  createEffect(() => {
    debouncedUrl();
    setPickedTag('');
  });

  createEffect(() => {
    const releases = previewData()?.releases ?? [];

    if (!releases.some((release) => release.tag_name === pickedTag())) {
      setPickedTag('');
    }
  });

  const selectedTag = createMemo(() =>
    resolveReleaseTag(previewData()?.releases ?? [], pickedTag()),
  );

  const releaseOptions = createMemo<ReleaseOption[]>(() => {
    const p = previewData();

    if (!p) {
      return [];
    }

    return p.releases.map((r) => ({
      value: r.tag_name,
      label: r.is_prerelease
        ? `${r.tag_name} (${t('plugins.prerelease')})`
        : r.tag_name,
    }));
  });

  const handleReleaseChange = (option: ReleaseOption | null) => {
    if (option) {
      setPickedTag(option.value);
    }
  };

  const handleInstall = async () => {
    const p = previewData();
    const tag = selectedTag();

    if (!p || !tag) {
      return;
    }

    const release = p.releases.find((r) => r.tag_name === tag);

    if (!release) {
      return;
    }

    try {
      await installMutation.mutateAsync({
        identifier: debouncedUrl(),
        downloadUrl: release.download_url,
        tagName: release.tag_name,
        version: release.version,
      });
      props.onInstalled?.();
    } catch {
      // Error handled by mutation
    }
  };

  const authorsStr = createMemo(() => {
    const authors = previewData()?.manifest?.authors;
    return authors ? authors.join(', ') : undefined;
  });

  return (
    <div class='gap-4 flex flex-col'>
      {/* URL input row */}
      <CombinedTextField
        class='flex-1'
        label={t('plugins.githubUrl')}
        inputProps={{
          placeholder: 'https://github.com/owner/repo',
        }}
        value={url()}
        onChange={handleUrlChange}
      />

      {/* Loading indicator */}
      <Show when={previewQuery.isLoading}>
        <p class='text-sm text-muted-foreground'>{t('common.loading')}</p>
      </Show>

      {/* Error message */}
      <Show when={previewQuery.isError}>
        <p class='text-sm text-destructive'>
          {previewQuery.error instanceof Error
            ? previewQuery.error.message
            : t('plugins.githubPreviewError')}
        </p>
      </Show>

      {/* Preview card */}
      <Show when={previewData()}>
        {(p) => (
          <PluginPreviewCard
            preview={p()}
            releaseOptions={releaseOptions()}
            selectedTag={selectedTag()}
            onReleaseChange={handleReleaseChange}
            onInstall={handleInstall}
            onClear={handleUrlClear}
            authors={authorsStr()}
            isInstalling={installMutation.isPending}
          />
        )}
      </Show>
    </div>
  );
};
