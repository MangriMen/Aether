import IconMdiAlertOutline from '~icons/mdi/alert-outline';
import { Show, type Component } from 'solid-js';

import { useUnrecognizedPluginDirs } from '@/features/plugins';
import { useTranslation } from '@/shared/i18n';

/** Tells why a folder put in the plugins folder by hand is not among the plugins. */
export const UnrecognizedPluginDirsNotice: Component = () => {
  const [{ t }] = useTranslation();

  const dirs = useUnrecognizedPluginDirs();

  return (
    <Show when={dirs.data?.length ? dirs.data : undefined}>
      {(names) => (
        <div
          role='alert'
          class={`
          mb-2 gap-2 p-3 flex items-start rounded-md border border-warning/50
          text-sm
        `}
        >
          <IconMdiAlertOutline class='mt-0.5 shrink-0 text-warning' />
          <span>
            {t('plugins.unrecognizedDirs', { names: names().join(', ') })}
          </span>
        </div>
      )}
    </Show>
  );
};
