import { open } from '@tauri-apps/plugin-dialog';

export const browseJavaInstallation = async () => {
  return open({
    title: 'Select javaw.exe',
    filters: [
      { name: 'Java Window Executable', extensions: ['exe'] },
      { name: 'All', extensions: ['*'] },
    ],
    multiple: false,
  });
};
