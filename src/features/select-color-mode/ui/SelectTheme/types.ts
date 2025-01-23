import { Option, ThemeConfig } from '@/shared/model';
import { SelectRootProps } from '@/shared/ui';

export type SelectThemeProps = Pick<
  SelectRootProps<Option<ThemeConfig>, never>,
  'name' | 'id'
>;
