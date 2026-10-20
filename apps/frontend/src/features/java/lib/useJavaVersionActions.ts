import { type Accessor } from 'solid-js';

import { closeDialog, showDialog } from '@/shared/dialogs';
import { logError } from '@/shared/log';

import type { JavaVersion, StrictJavaVersion } from '../model/javaVersion';

import { browseJavaInstallation } from '../api/browseJavaInstallation';
import {
  useActiveJavaInstallations,
  useDiscoverJava,
  useEditJava,
  useInstallJava,
} from '../api/javaQueries';
import { DetectJavaDialog } from '../ui/DetectJavaDialog';

const DETECT_JAVA_DIALOG_ID = 'detect-java';

export const useJavaVersionActions = (
  javaVersions: Accessor<JavaVersion[]>,
) => {
  const installJava = useInstallJava();
  const editJava = useEditJava();
  const discoverJava = useDiscoverJava();

  const activeInstallations = useActiveJavaInstallations();

  const checkIsInstalling = (majorVersion: number) => {
    return activeInstallations.data?.includes(majorVersion) ?? false;
  };

  // The mutations tell the user about a failure; the actions below only keep the
  // rejection from escaping into an event handler.
  const updateJavaPath = async (majorVersion: number, path: string) => {
    if (!path) {
      return false;
    }

    try {
      await editJava.mutateAsync({ majorVersion, path });
      return true;
    } catch (error) {
      logError('Failed to update Java path:', error);
      return false;
    }
  };

  const handleDetect = async (majorVersion: number) => {
    if (Number.isNaN(majorVersion)) {
      return;
    }

    const selectedVersionPath = () =>
      javaVersions().find((java) => java.majorVersion === majorVersion)?.path;

    let versions: Awaited<ReturnType<typeof discoverJava.mutateAsync>>;

    try {
      const discovered = await discoverJava.mutateAsync();
      versions = discovered.filter((v) => v.majorVersion === majorVersion);
    } catch (error) {
      logError('Failed to detect Java:', error);
      return;
    }

    const onSelect = async (newVersion: StrictJavaVersion) => {
      if (await updateJavaPath(newVersion.majorVersion, newVersion.path)) {
        closeDialog(DETECT_JAVA_DIALOG_ID);
      }
    };

    showDialog(DETECT_JAVA_DIALOG_ID, DetectJavaDialog, {
      versions,
      selectedVersionPath,
      onSelect,
      majorVersion,
    });
  };

  const handleBrowse = async (majorVersion: number) => {
    if (Number.isNaN(majorVersion)) {
      return;
    }

    try {
      const path = await browseJavaInstallation();

      if (typeof path === 'string') {
        await updateJavaPath(majorVersion, path);
      }
    } catch (error) {
      logError('Failed to browse for Java:', error);
    }
  };

  const installRecommended = async (majorVersion: number) => {
    try {
      await installJava.mutateAsync({ version: majorVersion, force: true });
    } catch (error) {
      logError('Failed to install Java:', error);
    }
  };

  return {
    installRecommended,
    isInstalling: checkIsInstalling,
    onDetect: handleDetect,
    onBrowse: handleBrowse,
  };
};
