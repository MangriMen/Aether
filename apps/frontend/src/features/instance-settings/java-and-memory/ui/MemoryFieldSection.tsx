import type { FormStore } from '@formisch/solid';

import { setInput, Field } from '@formisch/solid';
import { type Accessor, type Component, createMemo, onCleanup } from 'solid-js';

import { OverridableMemoryField } from '@/entities/settings';
import { debounce } from '@/shared/lib';

import type {
  JavaAndMemorySettingsSchemaInput,
  JavaAndMemorySettingsSchemaOutput,
  JavaAndMemorySettingsSchema,
} from '../model';

import { getDisplayValue } from '../lib';
import { MEMORY_SLIDER_HANDLE_DEBOUNCE } from '../model';

type Props = {
  form: FormStore<typeof JavaAndMemorySettingsSchema>;
  overridable?: boolean;
  defaultValues?: Accessor<
    Partial<JavaAndMemorySettingsSchemaInput> | undefined
  >;
  onChangePartial: (values: Partial<JavaAndMemorySettingsSchemaOutput>) => void;
};

export const MemoryFieldSection: Component<Props> = (props) => {
  const debouncedOnChange = createMemo(() =>
    debounce(props.onChangePartial, MEMORY_SLIDER_HANDLE_DEBOUNCE),
  );
  onCleanup(() => {
    debouncedOnChange().callAndClear();
  });

  return (
    <Field of={props.form} path={['overrideMemory']}>
      {(overrideMemory) => (
        <Field of={props.form} path={['memory', 'maximum']}>
          {(field) => (
            <OverridableMemoryField
              overridable={
                props.overridable && overrideMemory.input !== undefined
              }
              isOverridden={overrideMemory.input as boolean | undefined}
              value={getDisplayValue(
                props.overridable,
                field.input as number | undefined,
                overrideMemory.input as boolean | undefined,
                props.defaultValues?.()?.memory?.maximum,
              )}
              onChange={(value: number) => {
                setInput(props.form, {
                  path: ['memory', 'maximum'],
                  input: value,
                });
                debouncedOnChange()({ memory: { maximum: value } });
              }}
              onOverrideChange={(value) => {
                setInput(props.form, {
                  path: ['overrideMemory'],
                  input: value,
                });
                props.onChangePartial({ overrideMemory: value });
              }}
            />
          )}
        </Field>
      )}
    </Field>
  );
};
