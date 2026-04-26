import type { ImporterCapabilityEntry } from '../../../entities/plugins';
import type { ImporterTabConfig } from './importerTabConfig';

import { Image } from '../../../shared/ui';
import { ImportInstanceForm } from '../ui/ImportInstanceForm';

export const importerToTab = ({
  pluginId,
  capability: importer,
}: ImporterCapabilityEntry): ImporterTabConfig => {
  const key = `${pluginId}_${importer.id}`;

  return {
    value: key,
    label: importer.name,
    icon: () => <Image src={importer.icon ?? undefined} alt='' />,
    component: (props) => (
      <ImportInstanceForm pluginId={pluginId} importer={importer} {...props} />
    ),
  };
};
