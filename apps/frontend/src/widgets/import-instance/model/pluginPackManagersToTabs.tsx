import type {
  CapabilityEntryDto,
  PackManagerCapabilityMetadataDto,
} from '@/shared/api/bindings/instance';

import type {
  PackManagerTabConfig,
  PackManagerTabProps,
} from './packManagerTabConfig';

import { ImportInstanceForm } from '../ui/ImportInstanceForm';

export const packManagerToTab = ({
  pluginId,
  capability: manager,
}: CapabilityEntryDto<PackManagerCapabilityMetadataDto>): PackManagerTabConfig => {
  const key = `${pluginId}_${manager.id}`;

  return {
    value: key,
    label: manager.name,
    icon: manager.icon ?? undefined,
    component: (props: PackManagerTabProps) => (
      <ImportInstanceForm
        {...props}
        pluginId={pluginId}
        capabilityId={manager.id}
        fieldLabel={manager.fieldLabel}
        supportedExtensions={manager.supportedExtensions}
      />
    ),
  };
};
