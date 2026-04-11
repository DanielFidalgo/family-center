import React, { useState, useCallback } from 'react';
import {
  IonPage, IonContent, IonHeader, IonToolbar, IonTitle,
  IonButton, IonIcon, IonSpinner,
} from '@ionic/react';
import { chevronBackOutline, chevronForwardOutline, refreshOutline, addOutline } from 'ionicons/icons';
import { useHistory } from 'react-router-dom';
import dayjs from 'dayjs';

import { useSchedule, usePeople, useSettings, useRunSync } from '../api/hooks';
import WeekBoardComponent from '../components/schedule/WeekBoard';
import type { MergedEventGroup } from '@family-center/contracts';

function getWeekStart(date: Date, weekStartsMonday = true): Date {
  const d = new Date(date);
  d.setHours(0, 0, 0, 0);
  const day = d.getDay();
  const diff = weekStartsMonday
    ? (day === 0 ? -6 : 1 - day)
    : -day;
  d.setDate(d.getDate() + diff);
  return d;
}

function addDays(date: Date, days: number): Date {
  const d = new Date(date);
  d.setDate(d.getDate() + days);
  return d;
}

const WeekBoard: React.FC = () => {
  const history = useHistory();
  const { data: settings } = useSettings();
  const weekStartsMonday = settings?.weekStartsMonday ?? true;

  const [weekStart, setWeekStart] = useState(() => getWeekStart(new Date(), weekStartsMonday));
  const weekEnd = addDays(weekStart, 7);

  const { data: schedule, isLoading } = useSchedule(
    weekStart.toISOString(),
    weekEnd.toISOString()
  );
  const { data: people = [] } = usePeople();
  const syncMutation = useRunSync();

  const navigate = (delta: number) => {
    setWeekStart((prev) => addDays(prev, delta * 7));
  };

  const weekLabel = `${dayjs(weekStart).format('MMM D')} – ${dayjs(addDays(weekStart, 6)).format('MMM D, YYYY')}`;

  const handleEventClick = useCallback((event: MergedEventGroup) => {
    history.push(`/activity/${event.id}`);
  }, [history]);

  const handleDayClick = useCallback((date: Date) => {
    // Could navigate to day view — for now no-op or you can enable it
    // history.push('/day');
  }, []);

  return (
    <IonPage>
      <IonHeader>
        <IonToolbar>
          <IonButton slot="start" fill="clear" onClick={() => navigate(-1)}>
            <IonIcon icon={chevronBackOutline} />
          </IonButton>
          <IonTitle style={{ fontSize: '16px' }}>{weekLabel}</IonTitle>
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
          <IonButton slot="end" fill="clear" onClick={() => history.push('/activity/new')}>
            <IonIcon icon={addOutline} />
          </IonButton>
        </IonToolbar>
      </IonHeader>
      <IonContent>
        {isLoading ? (
          <div style={{ display: 'flex', justifyContent: 'center', padding: '40px' }}>
            <IonSpinner />
          </div>
        ) : (
          <WeekBoardComponent
            people={people}
            events={schedule?.events ?? []}
            activities={schedule?.localActivities ?? []}
            weekStart={weekStart}
            settings={settings}
            onEventClick={handleEventClick}
            onDayClick={handleDayClick}
          />
        )}
      </IonContent>
    </IonPage>
  );
};

export default WeekBoard;
