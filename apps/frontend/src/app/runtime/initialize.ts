import { applicationCommands } from '@/platform/ipc';
import { showWindow } from '@/platform/window';
import { logError } from '@/shared/lib';

import { exposeWindowMethods } from './exposeWindowMethods';

export const setupApp = async () => {
  try {
    exposeWindowMethods();

    await applicationCommands.waitForInitialization();

    await showWindow().catch(logError);

    await applicationCommands.initializePlugins();
  } catch (e) {
    logError(e);
    await showWindow().catch(logError);
    throw e;
  }
};
