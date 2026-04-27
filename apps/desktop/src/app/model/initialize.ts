import { applicationCommands } from '@/shared/api';
import { logError } from '@/shared/lib';

import { exposeWindowMethods } from '../lib';
import { showWindow } from './windowUtils';

export const setupApp = async () => {
  try {
    exposeWindowMethods();

    await applicationCommands.initializeState();

    await showWindow().catch(logError);

    await applicationCommands.initializePlugins();
  } catch (e) {
    logError(e);
    await showWindow().catch(logError);
    throw e;
  }
};
