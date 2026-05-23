import { createSignal } from 'solid-js';

import { useTestJre } from '@/entities/java';
import { isLauncherError } from '@/shared/model';

import type { JavaTestStatus } from '../model';

const DEFAULT_STATUS: JavaTestStatus = 'idle';

export const useJavaVersionTesting = () => {
  const [testingStatus, setTestingStatus] =
    createSignal<JavaTestStatus>(DEFAULT_STATUS);

  const testJava = useTestJre();

  const resetStatus = () => {
    setTestingStatus(DEFAULT_STATUS);
  };

  const test = async (majorVersion: number, path: string) => {
    if (Number.isNaN(majorVersion)) {
      return;
    }

    setTestingStatus('testing');
    try {
      const java = await testJava.mutateAsync(path);

      if (majorVersion !== java.majorVersion) {
        setTestingStatus('version-mismatch');
      } else {
        setTestingStatus('valid');
      }
    } catch (e) {
      if (
        isLauncherError(e) &&
        e.type === 'java' &&
        e.payload.code === 'INVALID_VERSION'
      ) {
        setTestingStatus('version-mismatch');
      } else {
        setTestingStatus('error');
      }
    }
  };

  return {
    test,
    testingStatus,
    resetStatus,
  };
};
