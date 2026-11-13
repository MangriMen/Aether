import { beforeEach, describe, expect, it, vi } from 'vitest';

const open = vi.hoisted(() => vi.fn());
const platform = vi.hoisted(() => vi.fn());

vi.mock('@tauri-apps/plugin-dialog', () => ({ open }));
vi.mock('@tauri-apps/plugin-os', () => ({ platform }));

import { browseJavaInstallation } from '../browseJavaInstallation';

const t = ((key: string) => `t:${key}`) as never;

describe('browseJavaInstallation', () => {
  beforeEach(() => {
    open.mockReset();
    platform.mockReset();
  });

  it('should_filter_by_exe_and_name_javaw_when_on_windows', async () => {
    platform.mockReturnValue('windows');

    await browseJavaInstallation(t);

    expect(open).toHaveBeenCalledWith({
      title: 't:javaVersion.browseTitleWindows',
      filters: [
        { name: 't:javaVersion.executableFilter', extensions: ['exe'] },
        { name: 't:common.allFiles', extensions: ['*'] },
      ],
      multiple: false,
    });
  });

  it('should_not_filter_by_exe_when_not_on_windows', async () => {
    platform.mockReturnValue('linux');

    await browseJavaInstallation(t);

    expect(open).toHaveBeenCalledWith({
      title: 't:javaVersion.browseTitle',
      filters: [{ name: 't:common.allFiles', extensions: ['*'] }],
      multiple: false,
    });
  });
});
