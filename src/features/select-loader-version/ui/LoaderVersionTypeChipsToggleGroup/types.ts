import { Option } from '@/shared/model';
import { ToggleGroupRootProps } from '@/shared/ui';

export type LoaderVersionTypeChipsToggleGroupProps = Exclude<
  ToggleGroupRootProps,
  'onChange'
> & {
  loaderTypes: Option[];
};
