import type { Option, ThemeConfig } from '@/shared/model';
import type { SelectRootProps } from '@/shared/ui';

export type SelectThemeProps = Pick<
  SelectRootProps<Option<ThemeConfig>, never>,
  'name' | 'id'
>;
