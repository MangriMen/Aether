import { Tabs, TabsContent, TabsList, TabsTrigger } from '../../../shared/ui';
import { ComponentShelf } from './ComponentShelf';

export const TabsLab = () => {
  return (
    <ComponentShelf title='Tabs'>
      <Tabs defaultValue='account' class='w-[400px]'>
        <TabsList class='grid w-full grid-cols-2'>
          <TabsTrigger value='account'>Account</TabsTrigger>
          <TabsTrigger value='password'>Password</TabsTrigger>
        </TabsList>
        <TabsContent value='account'>Account</TabsContent>
        <TabsContent value='password'>Password</TabsContent>
      </Tabs>
    </ComponentShelf>
  );
};
