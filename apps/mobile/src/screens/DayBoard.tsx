import React, { useState, useCallback } from 'react';
import {
  IonPage, IonContent, IonHeader, IonToolbar, IonTitle,
  IonButton, IonIcon, IonSpinner,
} from '@ionic/react';
import { chevronBackOutline, chevronForwardOutline, refreshOutline, addOutline } from 'ionicons/icons';
import { useHistory } from 'react-router-dom';
import dayjs from 'dayjs';

import { useSchedule } from '../api/hooks';
import { usePeople } from '../api/hooks';
import { useSettings } from '../api/hooks';
import { useRunSync } from '../api/hooks';
import DayTimeline from '../components/schedule/DayTimeline';
import type { MergedEventGroup, LocalActivity } from '@family-center/contracts';

function toIso(date: Date): string {
  return date.toISOString();
}

const DayBoard: React.FC = () => {
  const history = useHistory();
  const [date, setDate] = useState(() => {
    const d = new Date();
    d.setHours(0, 0, 0, 0);
    return d;
  });

  const dayEnd = new Date(date);
  dayEnd.setHours(23, 59, 59, 999);

  const { data: schedule, isLoading: scheduleLoading } = useSchedule(toIso(date), toIso(dayEnd));
  const { data: people = [] } = usePeople();
  const { data: settings } = useSettings();
  const syncMutation = useRunSync();

  const navigate = (delta: number) => {
    setDate((prev) => {
      const d = new Date(prev);
      d.setDate(d.getDate() + delta);
      return d;
    });
  };

  const isToday = () => {
    const today = new Date();
    today.setHours(0, 0, 0, 0);
    return date.getTime() === today.getTime();
  };

  const handleEventClick = useCallback((event: MergedEventGroup) => {
    history.push(`/activity/${event.id}`);
  }, [history]);

  const handleActivityClick = useCallback((activity: LocalActivity) => {
    history.push(`/activity/${activity.id}`);
  }, [history]);

  const dateLabel = isToday()
    ? 'Today'
    : dayjs(date).format('dddd, MMMM D');

  return (
    <IonPage>
      <IonHeader>
        <IonToolbar>
          <IonButton slot="start" fill="clear" onClick={() => navigate(-1)}>
            <IonIcon icon={chevronBackOutline} />
          </IonButton>
          <IonTitle style={{ fontSize: '18px' }}>
            {dateLabel}
          </IonTitle>
          <IonButton slot="end" fill="clear" onClick={() => navigate(1)}>
            <IonIcon icon={chevronForwardOutline} />
          </IonButton>
          <IonButton
            slot="end"
            fill="clear"
            onClick={() => syncMutation.mutate({})}
            disabled={syncMutation.isPending}
          >
            {syncMutation.isPending ? <IonSpinner name="crescent" /> : <IonIcon icon={refreshOutline} />}
          </IonButton>
          <IonButton
            slot="end"
            fill="clear"
            onClick={() => history.push('/activity/new')}
          >
            <IonIcon icon={addOutline} />
          </IonButton>
        </IonToolbar>
      </IonHeader>
      <IonContent>
        {scheduleLoading ? (
          <div style={{ display: 'flex', justifyContent: 'center', padding: '40px' }}>
            <IonSpinner />
          </div>
        ) : (
          <DayTimeline
            people={people}
            events={schedule?.events ?? []}
            activities={schedule?.localActivities ?? []}
            date={date}
            settings={settings}
            onEventClick={handleEventClick}
            onActivityClick={handleActivityClick}
          />
        )}
      </IonContent>
    </IonPage>
  );
};

export default DayBoard;
