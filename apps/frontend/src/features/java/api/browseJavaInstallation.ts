import { open } from '@tauri-apps/plugin-dialog';
import { platform as getOsPlatform } from '@tauri-apps/plugin-os';

import type { TFunction } from '@/shared/i18n';

export const browseJavaInstallation = async (t: TFunction) => {
  const isWindows = getOsPlatform() === 'windows';

  return open({
    title: isWindows
      ? t('javaVersion.browseTitleWindows')
      : t('javaVersion.browseTitle'),
    filters: [
      ...(isWindows
        ? [
            {
              name: t('javaVersion.executableFilter') ?? '',
              extensions: ['exe'],
            },
          ]
        : []),
      { name: t('common.allFiles') ?? '', extensions: ['*'] },
    ],
    multiple: false,
  });
};
