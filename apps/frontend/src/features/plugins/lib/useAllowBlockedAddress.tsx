import { useQueryClient } from '@tanstack/solid-query';
import { onCleanup } from 'solid-js';

import { registerErrorAction } from '@/features/errors';
import { askConfirmation } from '@/shared/dialogs';
import { useTranslation } from '@/shared/i18n';

import { useGrantPluginLocalHostAccess } from '../api/pluginQueries';
import { pluginsCache } from '../api/pluginsCache';
import { AllowHostButton } from '../ui/AllowHostButton';

const MAX_PORT = 65_535;

/** Four dotted numbers: how a URL writes back every IPv4 host it parses. */
const IPV4_HOST = /^\d{1,3}(?:\.\d{1,3}){3}$/;

/** An address literal or `localhost` itself: a host that stays the one it names. */
const isAddressOrLocalhost = (host: string) =>
  host === 'localhost' ||
  host === 'localhost.' ||
  host.startsWith('[') ||
  IPV4_HOST.test(host);

/**
 * The URL the local grant is given for `host` on `port`, or `undefined` when that URL would not
 * name exactly them (a `/`, `?`, `#`, `@` or `\` in the host, a port in it, a host not in
 * punycode, a port out of range): the confirmation would then name one address and the grant
 * give another. The grant keeps the URL's host and port only, so the scheme does not matter.
 */
const localGrantUrlOf = (host: string, port: number): string | undefined => {
  if (!Number.isInteger(port) || port < 1 || port > MAX_PORT) {
    return undefined;
  }

  const url = `http://${host}:${port}/`;

  try {
    const parsed = new URL(url);
    // `port` is empty for the scheme's own port; an IPv6 host keeps its brackets in `hostname`.
    const parsedPort = parsed.port === '' ? 80 : Number(parsed.port);

    return parsed.hostname === host && parsedPort === port ? url : undefined;
  } catch {
    return undefined;
  }
};

/**
 * A plugin's call failed because a host it asked for resolves to a local address (in the user's
 * network or on this computer): every error toast that says so offers to let the plugin reach
 * that host and port. Nothing is given before the user confirms a warning that says what it
 * gives. The plugin reads its hosts on every request, so nothing restarts. Called once, inside
 * the app's providers: the toast that shows the button is outside them.
 */
export const useAllowBlockedAddress = () => {
  const [{ t }] = useTranslation();
  const queryClient = useQueryClient();
  const grantLocalHostAccess = useGrantPluginLocalHostAccess();

  onCleanup(
    registerErrorAction((error) => {
      if (error.type !== 'plugin') {
        return undefined;
      }

      const { payload } = error;

      if (payload.code !== 'ADDRESS_BLOCKED') {
        return undefined;
      }

      const { pluginId: id, host, port } = payload.payload;
      const url = localGrantUrlOf(host, port);

      if (!url) {
        return undefined;
      }

      const texts = {
        name:
          pluginsCache.getData.detail(queryClient, id)?.manifest.metadata
            .name ?? id,
        address: `${host}:${port}`,
      };

      // A name is not tied to the address it has now: its owner can point it at another
      // address of the user's network later, so the user is told before allowing it.
      const nameNote = isAddressOrLocalhost(host)
        ? ''
        : ` ${t('pluginSettings.localHostAccessConfirmNameNote', texts)}`;

      const allow = async () => {
        const confirmed = await askConfirmation({
          header: t('pluginSettings.localHostAccessConfirmTitle', texts),
          description: `${t(
            'pluginSettings.localHostAccessConfirmDescription',
            texts,
          )}${nameNote}`,
          allow: t('pluginSettings.accessConfirmAllow'),
          deny: t('pluginSettings.accessConfirmDeny'),
        });

        if (!confirmed) {
          throw new Error('The user did not allow the local address');
        }

        return grantLocalHostAccess.mutateAsync({ id, url, dryRun: false });
      };

      return () => (
        <AllowHostButton
          label={t('pluginSettings.allowLocalAddress') ?? texts.address}
          onAllow={allow}
        />
      );
    }),
  );
};
