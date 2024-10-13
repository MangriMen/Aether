import { Option } from '@/shared/model';
import { ToggleGroupRootProps } from '@/shared/ui';

import { ModLoader } from '@/entities/minecraft';

export type SelectLoaderChipsProps = ToggleGroupRootProps & {
  loaders: Option<ModLoader>[];
};
