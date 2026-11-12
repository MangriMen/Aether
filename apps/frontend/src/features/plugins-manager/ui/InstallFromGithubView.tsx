import {
  createEffect,
  createMemo,
  createSignal,
  Show,
  type Component,
} from 'solid-js';

import { getTranslatedError, isLauncherError } from '@/features/errors';
import {
  useInstallPluginFromProvider,
  usePluginPreview,
} from '@/features/plugins';
import { useTranslation } from '@/shared/i18n';
import { CombinedTextField } from '@/shared/ui-kit';
import { debounce } from '@/shared/utils';

import type { ProviderPluginPreview } from '../model/pluginProvider';
import type { ReleaseOption } from './PluginPreviewCard';

import { PluginPreviewCard } from './PluginPreviewCard';

export type InstallFromGithubViewProps = {
  onInstalled?: () => void;
};

const DEBOUNCE_MS = 500;

/** This view is the one for GitHub, so it names its provider itself. */
const PROVIDER = 'git_hub' as const;

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

  // The tag the user asked for. Null until they pick one: the first preview is then
  // requested without a tag and names the release it read.
  const [requestedTag, setRequestedTag] = createSignal<string | null>(null);

  const previewQuery = usePluginPreview(PROVIDER, debouncedUrl, requestedTag);
  const installMutation = useInstallPluginFromProvider();

  // Only the preview of the tag in question: no data while another tag's is loading.
  const previewData = createMemo<ProviderPluginPreview | undefined>(
    () => previewQuery.data,
  );

  // The release list outlives the preview of a single tag, so that it stays on screen
  // while another tag loads. It belongs to the repository it was read from.
  const [releases, setReleases] = createSignal<
    ProviderPluginPreview['releases']
  >([]);

  let previousUrl: string | undefined;

  createEffect(() => {
    const url = debouncedUrl();

    if (url !== previousUrl) {
      previousUrl = url;
      setRequestedTag(null);
      setReleases([]);
    }
  });

  createEffect(() => {
    const data = previewData();

    if (data) {
      setReleases(data.releases);
    }
  });

  const selectedTag = createMemo(
    () => requestedTag() ?? previewData()?.tag ?? '',
  );

  // What may be installed: the preview that was read for the selected tag, nothing else.
  const readyPreview = createMemo(() => {
    const data = previewData();

    return data && data.tag === selectedTag() && !previewQuery.isFetching
      ? data
      : undefined;
  });

  const releaseOptions = createMemo<ReleaseOption[]>(() => {
    return releases().map((r) => ({
      value: r.tag_name,
      label: r.is_prerelease
        ? `${r.tag_name} (${t('plugins.prerelease')})`
        : r.tag_name,
    }));
  });

  const handleReleaseChange = (option: ReleaseOption | null) => {
    if (option) {
      setRequestedTag(option.value);
    }
  };

  const handleInstall = async () => {
    const p = readyPreview();

    if (!p) {
      return;
    }

    try {
      await installMutation.mutateAsync({
        sourceType: PROVIDER,
        identifier: debouncedUrl(),
        tagName: p.tag,
        archiveSha256: p.archive_sha256,
      });
      props.onInstalled?.();
    } catch {
      // Error handled by mutation
    }
  };

  const authorsStr = createMemo(() => {
    const authors = readyPreview()?.manifest.authors;
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
      <Show when={previewQuery.isLoading && releases().length === 0}>
        <p class='text-sm text-muted-foreground'>{t('common.loading')}</p>
      </Show>

      {/* Error message */}
      <Show when={previewQuery.isError}>
        <p class='text-sm text-destructive'>
          {isLauncherError(previewQuery.error)
            ? getTranslatedError(previewQuery.error, t)
            : t('plugins.githubPreviewError')}
        </p>
      </Show>

      {/* Preview card: stays up while another tag loads, so the releases can be picked */}
      <Show when={releases().length > 0 || previewData()}>
        <PluginPreviewCard
          preview={readyPreview()}
          isPreviewLoading={previewQuery.isFetching}
          releaseOptions={releaseOptions()}
          selectedTag={selectedTag()}
          onReleaseChange={handleReleaseChange}
          onInstall={handleInstall}
          onClear={handleUrlClear}
          authors={authorsStr()}
          isInstalling={installMutation.isPending}
        />
      </Show>
    </div>
  );
};
