import { Switch, SwitchControl, SwitchLabel, SwitchThumb } from '@/shared/ui';

import { ComponentShelf } from './ComponentShelf';

export const SwitchLab = () => {
  return (
    <ComponentShelf title='Switch'>
      <Switch class='space-x-2 flex items-center'>
        <SwitchControl>
          <SwitchThumb />
        </SwitchControl>
        <SwitchLabel>Airplane Mode</SwitchLabel>
      </Switch>
    </ComponentShelf>
  );
};
