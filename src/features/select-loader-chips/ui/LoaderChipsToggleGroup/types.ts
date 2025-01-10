import { Option } from '@/shared/model';
import { ToggleGroupRootProps } from '@/shared/ui';

import { ModLoader } from '@/entities/minecraft';

export type LoaderChipsToggleGroupProps = ToggleGroupRootProps & {
  loaders: Option<ModLoader>[];
  onChange: (value: ModLoader) => void;
};
