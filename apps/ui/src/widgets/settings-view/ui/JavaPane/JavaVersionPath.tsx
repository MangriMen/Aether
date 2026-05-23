import { platform as getOsPlatform } from '@tauri-apps/plugin-os';
import {
  createEffect,
  createMemo,
  createSignal,
  splitProps,
  untrack,
  type Component,
  type JSX,
} from 'solid-js';

import type { CombinedTextFieldProps } from '@/shared/ui';

import { useEditJava, useRemoveJava } from '@/entities/java';
import { CombinedTextField } from '@/shared/ui';

export type JavaVersionPathProps = CombinedTextFieldProps & {
  majorVersion: number;
};

const JAVA_PATH_EXAMPLES = {
  windows: 'C:\\Program Files\\Java\\jdk-21\\bin\\javaw.exe',
  unix: '/usr/lib/jvm/java-21-openjdk/bin/javaw',
} as const;

export const JavaVersionPath: Component<JavaVersionPathProps> = (props) => {
  const [local, others] = splitProps(props, ['value', 'majorVersion']);

  const editJava = useEditJava();
  const removeJava = useRemoveJava();

  const [value, setValue] = createSignal('');

  const applyPathChange = async (path: string) => {
    const trimmedPath = path.trim();
    const currentValue = (local.value ?? '').trim();

    if (trimmedPath === currentValue) {
      return;
    }

    if (Number.isNaN(local.majorVersion)) {
      return;
    }

    if (!trimmedPath) {
      try {
        await removeJava.mutateAsync(local.majorVersion);
      } catch {
        setValue(local.value ?? '');
      }
    } else {
      try {
        await editJava.mutateAsync({
          majorVersion: local.majorVersion,
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

  const pathPlaceholder = createMemo(() => {
    const platform = getOsPlatform();

    return platform === 'windows'
      ? JAVA_PATH_EXAMPLES.windows
      : JAVA_PATH_EXAMPLES.unix;
  });

  return (
    <CombinedTextField
      value={value()}
      onChange={setValue}
      inputProps={{
        type: 'text',
        placeholder: pathPlaceholder(),
        onKeyDown: handleKeyDown,
        onBlur: handleBlur,
      }}
      {...others}
    />
  );
};
