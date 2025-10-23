import type { Importer } from '@/entities/plugins';

import { Image } from '@/shared/ui';

import type { ImporterTabConfig } from './importerTabConfig';

import { ImportInstanceForm } from '../ui/ImportInstanceForm';

export const importerToTab = ({
  pluginId,
  capability: importer,
}: Importer): ImporterTabConfig => {
  const key = `${pluginId}_${importer.id}`;

  return {
    value: key,
    label: importer.name,
    icon: () => <Image src={importer.icon} alt='' />,
    component: (props) => (
      <ImportInstanceForm
        pluginId={pluginId}
        importer={importer}
        footerButtons={props.footerButtons}
      />
    ),
  };
};
