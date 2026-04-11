import React from 'react';
import { IonTabBar, IonTabButton, IonIcon, IonLabel } from '@ionic/react';
import { useIonRouter } from '@ionic/react';
import { calendarOutline, peopleOutline, settingsOutline, todayOutline } from 'ionicons/icons';

const TabBar: React.FC = () => {
  const router = useIonRouter();

  const tabs = [
    { path: '/day', label: 'Today', icon: todayOutline },
    { path: '/week', label: 'Week', icon: calendarOutline },
    { path: '/people', label: 'People', icon: peopleOutline },
    { path: '/settings', label: 'Settings', icon: settingsOutline },
  ];

  return (
    <IonTabBar slot="bottom" style={{ height: '56px' }}>
      {tabs.map(({ path, label, icon }) => (
        <IonTabButton
          key={path}
          tab={path.slice(1)}
          href={path}
          selected={router.routeInfo.pathname === path}
          style={{ minHeight: '56px' }}
        >
          <IonIcon icon={icon} />
          <IonLabel>{label}</IonLabel>
        </IonTabButton>
      ))}
    </IonTabBar>
  );
};

export default TabBar;
