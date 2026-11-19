import { onCleanup } from 'solid-js';

import { registerErrorAction } from '@/features/errors';
import { useTranslation } from '@/shared/i18n';

import { useGrantPluginHostAccess } from '../api/pluginQueries';
import { AllowHostButton } from '../ui/AllowHostButton';

/**
 * The URL the grant is given for `host`, or `undefined` when that URL would not name exactly
 * `host` (a `/`, `?`, `#` or `\` in it, a port, a host not in punycode): the button would then
 * say one host and allow another.
 */
const grantUrlOf = (host: string): string | undefined => {
  const url = `https://${host}/`;

  try {
    // `hostname`, not `host`: `host` keeps a port, and an IPv6 host keeps its brackets in both.
    return new URL(url).hostname === host ? url : undefined;
  } catch {
    return undefined;
  }
};

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

      const { pluginId: id, host } = payload.payload;
      const url = grantUrlOf(host);

      if (!url) {
        return undefined;
      }

      return () => (
        <AllowHostButton
          label={t('pluginSettings.allowHost', { host }) ?? host}
          onAllow={() =>
            grantHostAccess.mutateAsync({ id, url, dryRun: false })
          }
        />
      );
    }),
  );
};
