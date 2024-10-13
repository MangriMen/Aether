import { ConfigColorMode } from '@kobalte/core';

import { Option } from '@/shared/model';
import { SelectRootProps } from '@/shared/ui';

export type SelectColorModeProps = Pick<
  SelectRootProps<Option<ConfigColorMode>, never>,
  'name' | 'id'
>;
