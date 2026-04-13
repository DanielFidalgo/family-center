import React, { useState, useCallback } from 'react';
import {
  IonPage, IonContent, IonHeader, IonToolbar,
  IonButton, IonIcon, IonSpinner,
} from '@ionic/react';
import { chevronBackOutline, chevronForwardOutline, refreshOutline, addOutline } from 'ionicons/icons';
import { useHistory, useLocation } from 'react-router-dom';
import dayjs from 'dayjs';

import { useSchedule, usePeople, useSettings, useRunSync } from '../api/hooks';
import DayTimeline from '../components/schedule/DayTimeline';
import type { MergedEventGroup, LocalActivity } from '@family-center/contracts';

function toIso(date: Date): string {
  return date.toISOString();
}

const DayBoard: React.FC = () => {
  const history = useHistory();
  const location = useLocation<{ date?: string }>();

  const [date, setDate] = useState<Date>(() => {
    // Allow week view to pass in a specific date
    if (location.state?.date) {
      const d = new Date(location.state.date);
      d.setHours(0, 0, 0, 0);
      return d;
    }
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

  const isToday = (() => {
    const t = new Date();
    t.setHours(0, 0, 0, 0);
    return date.getTime() === t.getTime();
  })();

  const goToToday = () => {
    const d = new Date();
    d.setHours(0, 0, 0, 0);
    setDate(d);
  };

  const handleEventClick = useCallback((event: MergedEventGroup) => {
    history.push(`/activity/${event.id}`);
  }, [history]);

  const handleActivityClick = useCallback((activity: LocalActivity) => {
    history.push(`/activity/${activity.id}`, { date: date.toISOString() });
  }, [history, date]);

  const dateLabel = isToday ? 'Today' : dayjs(date).format('ddd, MMM D');

  return (
    <IonPage>
      <IonHeader>
        <IonToolbar>
          <IonButton slot="start" fill="clear" onClick={() => navigate(-1)}>
            <IonIcon icon={chevronBackOutline} />
          </IonButton>
          <div
            slot="start"
            style={{ display: 'flex', flexDirection: 'column', paddingLeft: '4px' }}
          >
            <span style={{
              fontFamily: 'var(--fc-font-display)',
              fontSize: '17px',
              fontWeight: 700,
              color: isToday ? 'var(--fc-accent)' : 'var(--fc-text-primary)',
              letterSpacing: '-0.01em',
            }}>
              {dateLabel}
            </span>
            {!isToday && (
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
        {scheduleLoading ? (
          <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100%' }}>
            <IonSpinner />
          </div>
        ) : (
          <DayTimeline
            people={people}
            events={schedule?.events ?? []}
            activities={schedule?.localActivities ?? []}
            completions={schedule?.completions ?? []}
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
