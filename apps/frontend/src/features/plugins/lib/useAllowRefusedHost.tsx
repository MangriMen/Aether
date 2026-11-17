import { onCleanup } from 'solid-js';

import { registerErrorAction } from '@/features/errors';
import { useTranslation } from '@/shared/i18n';

import { useGrantPluginHostAccess } from '../api/pluginQueries';
import { AllowHostButton } from '../ui/AllowHostButton';

/**
 * A plugin's call failed because a host it asked for is not among its allowed hosts: every
 * error toast that says so offers to allow that host. The grant takes a URL and keeps only its
 * host, and the plugin reads its hosts on every request, so nothing restarts. Called once,
 * inside the app's providers: the toast that shows the button is outside them.
 */
export const useAllowRefusedHost = () => {
  const [{ t }] = useTranslation();
  const grantHostAccess = useGrantPluginHostAccess();

  onCleanup(
    registerErrorAction((error) => {
      if (error.type !== 'plugin') {
        return undefined;
      }

      const { payload } = error;

      if (payload.code !== 'HOST_NOT_ALLOWED') {
        return undefined;
      }

      const { plugin_id: id, host } = payload.payload;

      return () => (
        <AllowHostButton
          label={t('pluginSettings.allowHost', { host }) ?? host}
          onAllow={() =>
            grantHostAccess.mutateAsync({
              id,
              // An IPv6 host comes in brackets already, as a URL writes it.
              url: `https://${host}/`,
              dryRun: false,
            })
          }
        />
      );
    }),
  );
};
