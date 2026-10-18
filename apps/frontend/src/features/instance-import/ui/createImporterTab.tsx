import type { ImporterCapabilityEntry } from '@/features/plugins';

import { Image } from '@/shared/ui-kit';

import type { ImporterTabConfig } from '../model/importerTabConfig';

import { ImportInstanceForm } from './ImportInstanceForm';

export const createImporterTab = ({
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
