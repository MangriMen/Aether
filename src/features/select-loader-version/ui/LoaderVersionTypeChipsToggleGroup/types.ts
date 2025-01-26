import type { Option } from '@/shared/model';
import type { ToggleGroupRootProps } from '@/shared/ui';

export type LoaderVersionTypeChipsToggleGroupProps = Exclude<
  ToggleGroupRootProps,
  'onChange'
> & {
  loaderTypes: Option[];
};
