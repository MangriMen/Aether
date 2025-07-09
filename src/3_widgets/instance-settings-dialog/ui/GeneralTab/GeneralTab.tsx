import { throttle } from '@solid-primitives/scheduled';
import type { Component, ComponentProps } from 'solid-js';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { CombinedTextField } from '@/shared/ui';

import { useEditInstance, type Instance } from '@/entities/instances';

import { useTranslation } from '@/shared/model';
import type { InstanceSettingsTabProps } from '../../model';

export type GeneralTabProps = ComponentProps<'div'> & InstanceSettingsTabProps;

export const GeneralTab: Component<GeneralTabProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const [{ t }] = useTranslation();

  const { mutateAsync: editInstance } = useEditInstance();

  const handleChangeNameThrottle = throttle(
    (id: Instance['id'], value: string) => {
      editInstance({ id, edit: { name: value } });
    },
    16,
  );

  const handleChangeName = (value: string) => {
    handleChangeNameThrottle(local.instance.id, value);
  };

  return (
    <div class={cn('flex flex-col', local.class)} {...others}>
      <CombinedTextField
        label={t('common.name')}
        defaultValue={local.instance.name}
        onChange={handleChangeName}
      />
    </div>
  );
};
