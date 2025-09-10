import type { PartialValues } from '@modular-forms/solid';
import {
  splitProps,
  type Accessor,
  type Component,
  type ComponentProps,
} from 'solid-js';
import type {
  HooksSettingsSchemaValuesInput,
  HooksSettingsSchemaValuesOutput,
} from '../model';
import { useHooksSettingsForm, useResetHooksSettingsFormValues } from '../lib';
import { cn } from '@/shared/lib';
import { CombinedTextField } from '@/shared/ui';

export type HooksSettingsFormProps = Omit<
  ComponentProps<'form'>,
  'onSubmit' | 'children'
> & {
  initialValues: Accessor<
    PartialValues<HooksSettingsSchemaValuesInput> | undefined
  >;
  onChangePartial?: (values: Partial<HooksSettingsSchemaValuesOutput>) => void;
};

export const HooksSettingsForm: Component<HooksSettingsFormProps> = (props) => {
  const [local, others] = splitProps(props, [
    'initialValues',
    'onChangePartial',
    'class',
  ]);

  const [form, { Form }] = useHooksSettingsForm();
  useResetHooksSettingsFormValues(form, () => local.initialValues());

  return (
    <Form class={cn('flex flex-col gap-2', local.class)} {...others}>
      <span class='text-lg font-medium'>Hooks</span>
      <div class='flex flex-col'>
        <span class='font-medium text-muted-foreground brightness-125'>
          Pre launch
        </span>
        <CombinedTextField
          label={
            <span class='text-base text-muted-foreground'>
              Ran before the instance is launched
            </span>
          }
          inputProps={{ type: 'text', placeholder: 'Enter pre-launch command' }}
        />
      </div>
      <div class='flex flex-col'>
        <span class='font-medium text-muted-foreground brightness-125'>
          Wrapper
        </span>
        <CombinedTextField
          label={
            <span class='text-base text-muted-foreground'>
              Wrapper command for launching Minecraft
            </span>
          }
          inputProps={{ type: 'text', placeholder: 'Enter wrapper command' }}
        />
      </div>
      <div class='flex flex-col'>
        <span class='font-medium text-muted-foreground brightness-125'>
          Post exit
        </span>
        <CombinedTextField
          label={
            <span class='text-base text-muted-foreground'>
              Ran after the game closes
            </span>
          }
          inputProps={{ type: 'text', placeholder: 'Enter post-exit command' }}
        />
      </div>
    </Form>
  );
};
