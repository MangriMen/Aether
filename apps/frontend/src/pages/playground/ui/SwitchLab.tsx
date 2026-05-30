import { Switch, SwitchControl, SwitchLabel, SwitchThumb } from '@/shared/ui';

import { ComponentShelf } from './ComponentShelf';

export const SwitchLab = () => {
  return (
    <ComponentShelf title='Switch'>
      <Switch class='flex items-center space-x-2'>
        <SwitchControl>
          <SwitchThumb />
        </SwitchControl>
        <SwitchLabel>Airplane Mode</SwitchLabel>
      </Switch>
    </ComponentShelf>
  );
};
