import React, { useState, useCallback } from 'react';
import {
  IonPage, IonContent, IonHeader, IonToolbar,
  IonButton, IonIcon, IonSpinner,
} from '@ionic/react';
import { chevronBackOutline, chevronForwardOutline, refreshOutline, addOutline } from 'ionicons/icons';
import { useHistory } from 'react-router-dom';
import dayjs from 'dayjs';

import { useSchedule, usePeople, useSettings, useRunSync } from '../api/hooks';
import WeekBoardComponent from '../components/schedule/WeekBoard';
import type { MergedEventGroup, LocalActivity } from '@family-center/contracts';

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

  const goToToday = () => {
    setWeekStart(getWeekStart(new Date(), weekStartsMonday));
  };

  const weekLabel = `${dayjs(weekStart).format('MMM D')} – ${dayjs(addDays(weekStart, 6)).format('MMM D')}`;

  const isCurrentWeek = (() => {
    const thisWeekStart = getWeekStart(new Date(), weekStartsMonday);
    return weekStart.getTime() === thisWeekStart.getTime();
  })();

  const handleEventClick = useCallback((event: MergedEventGroup) => {
    history.push(`/activity/${event.id}`);
  }, [history]);

  const handleActivityClick = useCallback((activity: LocalActivity, date: Date) => {
    history.push(`/activity/${activity.id}`, { date: date.toISOString() });
  }, [history]);

  const handleDayClick = useCallback((date: Date) => {
    history.push('/day', { date: date.toISOString() });
  }, [history]);

  return (
    <IonPage>
      <IonHeader>
        <IonToolbar>
          <IonButton slot="start" fill="clear" onClick={() => navigate(-1)}>
            <IonIcon icon={chevronBackOutline} />
          </IonButton>
          <div
            slot="start"
            style={{
              display: 'flex',
              flexDirection: 'column',
              paddingLeft: '4px',
            }}
          >
            <span style={{
              fontFamily: 'var(--fc-font-display)',
              fontSize: '17px',
              fontWeight: 700,
              color: 'var(--fc-text-primary)',
              letterSpacing: '-0.01em',
            }}>
              {weekLabel}
            </span>
            {!isCurrentWeek && (
              <button
                onClick={goToToday}
                style={{
                  background: 'none',
                  border: 'none',
                  color: 'var(--fc-accent)',
                  fontSize: '11px',
                  fontFamily: 'var(--fc-font-body)',
                  fontWeight: 600,
                  cursor: 'pointer',
                  padding: 0,
                  textAlign: 'left',
                }}
              >
                Back to today
              </button>
            )}
          </div>
          <IonButton slot="end" fill="clear" onClick={() => navigate(1)}>
            <IonIcon icon={chevronForwardOutline} />
          </IonButton>
          <IonButton
            slot="end"
            fill="clear"
            onClick={() => syncMutation.mutate({})}
            disabled={syncMutation.isPending}
          >
            {syncMutation.isPending
              ? <IonSpinner name="crescent" style={{ width: 18, height: 18 }} />
              : <IonIcon icon={refreshOutline} />}
          </IonButton>
          <IonButton slot="end" fill="clear" onClick={() => history.push('/activity/new')}>
            <IonIcon icon={addOutline} />
          </IonButton>
        </IonToolbar>
      </IonHeader>
      <IonContent scrollY={false}>
        {isLoading ? (
          <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100%' }}>
            <IonSpinner />
          </div>
        ) : (
          <WeekBoardComponent
            people={people}
            events={schedule?.events ?? []}
            activities={schedule?.localActivities ?? []}
            completions={schedule?.completions ?? []}
            weekStart={weekStart}
            settings={settings}
            onEventClick={handleEventClick}
            onActivityClick={handleActivityClick}
            onDayClick={handleDayClick}
          />
        )}
      </IonContent>
    </IonPage>
  );
};

export default WeekBoard;
