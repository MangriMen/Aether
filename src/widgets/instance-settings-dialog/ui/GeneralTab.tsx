import { throttle } from '@solid-primitives/scheduled';
import type { Component, ComponentProps } from 'solid-js';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { CombinedTextField } from '@/shared/ui';

import type { Instance, InstanceSettingsTabProps } from '@/entities/instances';
import { editInstance } from '@/entities/instances';

import { useTranslate } from '@/shared/model';

export type GeneralTabProps = ComponentProps<'div'> & InstanceSettingsTabProps;

export const GeneralTab: Component<GeneralTabProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const [{ t }] = useTranslate();

  const handleChangeNameThrottle = throttle(
    (id: Instance['id'], value: string) => {
      editInstance(id, { name: value });
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
