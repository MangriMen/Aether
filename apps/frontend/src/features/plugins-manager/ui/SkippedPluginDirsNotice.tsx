import IconMdiAlertOutline from '~icons/mdi/alert-outline';
import { For, Show, type Component } from 'solid-js';

import {
  useSkippedPluginDirs,
  type SkippedPluginDir,
} from '@/features/plugins';
import { useTranslation } from '@/shared/i18n';

/** Tells why a folder in the plugins folder is not among the plugins. */
export const SkippedPluginDirsNotice: Component = () => {
  const [{ t }] = useTranslation();

  const dirs = useSkippedPluginDirs();

  const reason = (dir: SkippedPluginDir) =>
    dir.reason.code === 'INVALID_NAME'
      ? t('plugins.skippedDirs.invalidName')
      : t('plugins.skippedDirs.loadFailed', {
          details: dir.reason.payload.details,
        });

  return (
    <Show when={dirs.data?.length ? dirs.data : undefined}>
      {(skipped) => (
        <div
          role='alert'
          class={`
            mb-2 gap-2 p-3 flex items-start rounded-md border
            border-warning/50 text-sm
          `}
        >
          <IconMdiAlertOutline class='mt-0.5 shrink-0 text-warning' />
          <div class='gap-1 flex flex-col'>
            <span>{t('plugins.skippedDirs.title')}</span>
            <ul class='list-disc pl-4'>
              <For each={skipped()}>
                {(dir) => (
                  <li>
                    <b>{dir.dir_name}</b>: {reason(dir)}
                  </li>
                )}
              </For>
            </ul>
          </div>
        </div>
      )}
    </Show>
  );
};
