import { invoke } from '@tauri-apps/api/core';
import type { ActionOnInstanceLaunchType } from '../model';

export const getActionOnInstanceLaunch = () =>
  invoke<ActionOnInstanceLaunchType>('get_action_on_instance_launch');

export const setActionOnInstanceLaunch = (
  actionOnInstanceLaunch: ActionOnInstanceLaunchType,
) => invoke('set_action_on_instance_launch', { actionOnInstanceLaunch });
