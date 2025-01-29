import type { Option } from '@/shared/model';
import type { ToggleGroupRootProps } from '@/shared/ui';

import type { ModLoader } from '@/entities/minecrafts';

export type LoaderChipsToggleGroupProps = ToggleGroupRootProps & {
  loaders: Option<ModLoader>[];
  onChange: (value: ModLoader) => void;
};
