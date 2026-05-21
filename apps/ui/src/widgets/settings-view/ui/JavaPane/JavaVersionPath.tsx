import {
  createEffect,
  createSignal,
  splitProps,
  untrack,
  type Component,
  type JSX,
} from 'solid-js';

import type { CombinedTextFieldProps } from '@/shared/ui';

import { useEditJava, useRemoveJava } from '@/entities/java';
import { useTranslation } from '@/shared/model';
import { CombinedTextField } from '@/shared/ui';

import { parseJavaVersion } from '../../lib/parseJavaVersion';

export type JavaVersionPathProps = CombinedTextFieldProps & {
  majorVersion: string;
};

export const JavaVersionPath: Component<JavaVersionPathProps> = (props) => {
  const [local, others] = splitProps(props, ['value', 'majorVersion']);

  const [{ t }] = useTranslation();

  const editJava = useEditJava();
  const removeJava = useRemoveJava();

  const [value, setValue] = createSignal('');

  const applyPathChange = async (path: string) => {
    const trimmedPath = path.trim();
    const currentValue = (local.value ?? '').trim();

    if (trimmedPath === currentValue) {
      return;
    }

    const versionNum = parseJavaVersion(local.majorVersion);

    if (versionNum === undefined || Number.isNaN(versionNum)) {
      return;
    }

    if (!trimmedPath) {
      try {
        await removeJava.mutateAsync(versionNum);
      } catch {
        setValue(local.value ?? '');
      }
    } else {
      try {
        await editJava.mutateAsync({
          majorVersion: versionNum,
          path: trimmedPath,
        });
      } catch {
        setValue(local.value ?? '');
      }
    }
  };

  const handleBlur: JSX.FocusEventHandler<HTMLInputElement, FocusEvent> = (
    e,
  ) => {
    applyPathChange(e.currentTarget.value);
  };

  const handleKeyDown: JSX.EventHandler<HTMLInputElement, KeyboardEvent> = (
    e,
  ) => {
    if (e.key === 'Enter') {
      e.currentTarget.blur();
    }
  };

  createEffect(() => {
    const remoteValue = local.value ?? '';
    untrack(() => {
      if (remoteValue !== value()) {
        setValue(remoteValue);
      }
    });
  });

  return (
    <CombinedTextField
      value={value()}
      onChange={setValue}
      inputProps={{
        type: 'text',
        placeholder: t('javaVersion.pathPlaceholder'),
        onKeyDown: handleKeyDown,
        onBlur: handleBlur,
      }}
      {...others}
    />
  );
};
