import type { PluginImporters } from '@/entities/plugins';

import { Image } from '@/shared/ui';

import type { ImporterTabConfig } from './importerTabConfig';

import { ImportInstanceForm } from '../ui/ImportInstanceForm';

export const pluginImportersToTabs = (
  pluginImporters: PluginImporters,
): ImporterTabConfig[] =>
  pluginImporters.importers.map((importer) => {
    const key = `${pluginImporters.plugin_id}_${importer.id}`;

    return {
      value: key,
      label: importer.name,
      icon: () => <Image src={importer.icon} alt='' />,
      component: (props) => (
        <ImportInstanceForm
          pluginId={pluginImporters.plugin_id}
          importer={importer}
          footerButtons={props.footerButtons}
        />
      ),
    };
  });
