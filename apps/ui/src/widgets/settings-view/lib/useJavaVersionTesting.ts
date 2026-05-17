import { createSignal } from 'solid-js';

import { useTestJava } from '@/entities/java';
import { isLauncherError } from '@/shared/model';

import type { JavaTestStatus } from '../model';

import { parseJavaVersion } from './parseJavaVersion';

export const useJavaVersionTesting = () => {
  const [testingStatus, setTestingStatus] = createSignal<
    JavaTestStatus | undefined
  >(undefined);

  const testJava = useTestJava();

  const resetStatus = () => {
    setTestingStatus(undefined);
  };

  const test = async (version: string, path: string) => {
    const versionNum = parseJavaVersion(version);

    if (!versionNum) {
      return;
    }

    setTestingStatus('testing');
    try {
      await testJava.mutateAsync({
        major_version: versionNum,
        path,
      });

      setTestingStatus('valid');
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
    } finally {
      if (testingStatus() === 'testing') {
        setTestingStatus(undefined);
      }
    }
  };

  return {
    test,
    testingStatus,
    resetStatus,
  };
};
