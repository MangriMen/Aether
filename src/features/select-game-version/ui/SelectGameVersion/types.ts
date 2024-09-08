import { SelectRootProps } from '@/shared/ui';

export type SelectGameVersionProps<Option> = SelectRootProps<
  Option,
  never,
  'div'
> & {
  advanced?: boolean;
};
