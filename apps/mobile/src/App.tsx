import React from 'react';
import { IonApp, IonRouterOutlet, setupIonicReact } from '@ionic/react';
import { IonReactRouter } from '@ionic/react-router';
import { Route, Switch, Redirect } from 'react-router-dom';

import DayBoard from './screens/DayBoard';
import WeekBoard from './screens/WeekBoard';
import Onboarding from './screens/Onboarding';
import PeopleManagement from './screens/PeopleManagement';
import GoogleAccounts from './screens/GoogleAccounts';
import CalendarSelection from './screens/CalendarSelection';
import ActivityDetail from './screens/ActivityDetail';
import CreateActivity from './screens/CreateActivity';
import SettingsScreen from './screens/SettingsScreen';

import { useAppStore } from './store/appStore';
import TabBar from './components/navigation/TabBar';

setupIonicReact({
  mode: 'md', // Material Design for Android wall display
});

const App: React.FC = () => {
  const isOnboarded = useAppStore((s) => s.isOnboarded);

  return (
    <IonApp>
      <IonReactRouter>
        {!isOnboarded ? (
          <IonRouterOutlet>
            <Route exact path="/onboarding" component={Onboarding} />
            <Redirect to="/onboarding" />
          </IonRouterOutlet>
        ) : (
          <>
            <IonRouterOutlet id="main">
              <Route exact path="/day" component={DayBoard} />
              <Route exact path="/week" component={WeekBoard} />
              <Route exact path="/people" component={PeopleManagement} />
              <Route exact path="/google-accounts" component={GoogleAccounts} />
              <Route exact path="/calendars/:accountId" component={CalendarSelection} />
              <Route exact path="/activity/:id" component={ActivityDetail} />
              <Route exact path="/activity/new" component={CreateActivity} />
              <Route exact path="/settings" component={SettingsScreen} />
              <Redirect exact from="/" to="/week" />
            </IonRouterOutlet>
            <TabBar />
          </>
        )}
      </IonReactRouter>
    </IonApp>
  );
};

export default App;
