import type { Component, ComponentProps } from 'solid-js';

import { open } from '@tauri-apps/plugin-dialog';
import IconMdiFileImport from '~icons/mdi/file-import';

import { useImportPlugins } from '@/entities/plugins';
import { useTranslation } from '@/shared/model';
import { CombinedTooltip, IconButton } from '@/shared/ui';

export type InstallFromPluginPackageButtonProps = ComponentProps<'div'>;

export const InstallFromPluginPackageButton: Component<
  InstallFromPluginPackageButtonProps
> = (props) => {
  const [{ t }] = useTranslation();

  const importPlugin = useImportPlugins();

  const handleSelectPackage = async () => {
    const path = await open({
      title: 'Import plugin',
      filters: [{ name: 'Plugin package', extensions: ['zip'] }],
      multiple: true,
    });

    if (!path) {
      return;
    }

    await importPlugin.mutateAsync(path);
  };

  return (
    <CombinedTooltip
      label={t('plugins.importFromPackage')}
      as={IconButton}
      variant='secondary'
      icon={IconMdiFileImport}
      onClick={handleSelectPackage}
      {...props}
    />
  );
};
