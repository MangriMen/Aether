import { type Accessor } from 'solid-js';

import { closeDialog, showDialog } from '@/shared/dialogs';
import { logError } from '@/shared/utils';

import type { JavaVersion, StrictJavaVersion } from '../model/javaVersion';

import {
  useActiveJavaInstallations,
  useDiscoverJava,
  useEditJava,
  useInstallJava,
} from '../api/javaQueries';
import { browseJavaInstallation } from '../model/browseJavaInstallation';
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

  const updateJavaPath = async (majorVersion: number, path: string) => {
    if (!path) {
      return;
    }

    try {
      await editJava.mutateAsync({ majorVersion, path });
    } catch (error) {
      logError('Failed to update Java path:', error);
      throw error;
    }
  };

  const handleDetect = async (majorVersion: number) => {
    if (Number.isNaN(majorVersion)) {
      return;
    }

    const selectedVersionPath = () =>
      javaVersions().find((java) => java.majorVersion === majorVersion)?.path;

    const discovered = await discoverJava.mutateAsync();
    const versions = discovered.filter((v) => v.majorVersion === majorVersion);

    const onSelect = async (newVersion: StrictJavaVersion) => {
      await updateJavaPath(newVersion.majorVersion, newVersion.path);
      closeDialog(DETECT_JAVA_DIALOG_ID);
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

    const path = await browseJavaInstallation();

    if (typeof path === 'string') {
      await updateJavaPath(majorVersion, path);
    }
  };

  const installRecommended = async (majorVersion: number) => {
    await installJava.mutateAsync({ version: majorVersion, force: true });
  };

  return {
    installRecommended,
    isInstalling: checkIsInstalling,
    onDetect: handleDetect,
    onBrowse: handleBrowse,
  };
};
