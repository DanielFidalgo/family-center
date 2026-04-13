import React from 'react';
import {
  IonPage, IonContent, IonHeader, IonToolbar, IonTitle,
  IonBackButton, IonButtons, IonSpinner,
} from '@ionic/react';
import { useParams, useLocation } from 'react-router-dom';
import dayjs from 'dayjs';

import { useActivities, useSchedule, useCompleteActivity, useUncompleteActivity } from '../api/hooks';
import type { ChoreCategory } from '@family-center/contracts';

const CATEGORY_LABELS: Record<ChoreCategory, string> = {
  cleaning: '🧹 Cleaning',
  kitchen: '🍳 Kitchen',
  laundry: '👕 Laundry',
  hygiene: '🚿 Hygiene',
  pets: '🐾 Pets',
  homework: '📚 Homework',
  exercise: '🏃 Exercise',
  errands: '🛒 Errands',
  other: '📋 Other',
};

const s: Record<string, React.CSSProperties> = {
  page: { display: 'flex', flexDirection: 'column', height: '100%', background: 'var(--fc-bg-primary)' },
  body: { padding: '24px 20px' },
  title: {
    fontFamily: 'var(--fc-font-display)',
    fontSize: '26px',
    fontWeight: 700,
    color: 'var(--fc-text-primary)',
    letterSpacing: '-0.02em',
    marginBottom: '4px',
  },
  titleDone: { textDecoration: 'line-through', opacity: 0.5 },
  doneChip: {
    display: 'inline-block',
    marginBottom: '12px',
    padding: '2px 10px',
    borderRadius: '100px',
    background: 'var(--fc-green-dim, #d4f5e2)',
    color: 'var(--fc-green, #22c55e)',
    fontSize: '11px',
    fontWeight: 700,
    letterSpacing: '0.06em',
    textTransform: 'uppercase' as const,
  },
  metaRow: { display: 'flex', gap: '8px', flexWrap: 'wrap' as const, marginBottom: '16px' },
  badge: {
    padding: '3px 10px',
    borderRadius: '100px',
    background: 'var(--fc-bg-secondary)',
    color: 'var(--fc-text-secondary)',
    fontSize: '12px',
    fontWeight: 600,
  },
  desc: {
    color: 'var(--fc-text-secondary)',
    fontSize: '15px',
    lineHeight: '1.6',
    marginBottom: '28px',
    whiteSpace: 'pre-wrap' as const,
  },
  divider: { borderBottom: '1px solid var(--fc-border)', marginBottom: '24px' },
  sectionLabel: {
    fontFamily: 'var(--fc-font-display)',
    fontSize: '12px',
    fontWeight: 700,
    letterSpacing: '0.08em',
    textTransform: 'uppercase' as const,
    color: 'var(--fc-text-secondary)',
    marginBottom: '12px',
  },
  dateLabel: {
    fontSize: '14px',
    color: 'var(--fc-text-primary)',
    fontWeight: 600,
    marginBottom: '16px',
  },
  doneBtn: {
    width: '100%',
    padding: '14px 24px',
    borderRadius: '14px',
    border: '2px solid var(--fc-green, #22c55e)',
    background: 'var(--fc-green, #22c55e)',
    color: '#fff',
    fontFamily: 'var(--fc-font-display)',
    fontSize: '15px',
    fontWeight: 700,
    cursor: 'pointer',
    letterSpacing: '-0.01em',
  },
  undoneBtn: {
    width: '100%',
    padding: '14px 24px',
    borderRadius: '14px',
    border: '2px solid var(--fc-border-strong)',
    background: 'transparent',
    color: 'var(--fc-text-secondary)',
    fontFamily: 'var(--fc-font-display)',
    fontSize: '15px',
    fontWeight: 600,
    cursor: 'pointer',
    letterSpacing: '-0.01em',
  },
};

const ActivityDetail: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const location = useLocation<{ date?: string }>();

  const date = location.state?.date
    ? dayjs(location.state.date).format('YYYY-MM-DD')
    : dayjs().format('YYYY-MM-DD');

  const dayStart = dayjs(date).startOf('day').toISOString();
  const dayEnd = dayjs(date).endOf('day').toISOString();

  const { data: activities = [], isLoading: activitiesLoading } = useActivities();
  const { data: schedule } = useSchedule(dayStart, dayEnd);

  const activity = activities.find((a) => a.id === id);
  const completion = schedule?.completions?.find(
    (c) => c.localActivityId === id && c.completedDate === date
  );
  const isDone = !!completion;

  const completeMutation = useCompleteActivity();
  const uncompleteMutation = useUncompleteActivity();

  const handleToggleDone = () => {
    if (isDone) {
      uncompleteMutation.mutate({ id, date });
    } else {
      completeMutation.mutate({ id, date });
    }
  };

  const isPending = completeMutation.isPending || uncompleteMutation.isPending;

  const isChore = !!activity?.recurrenceRule;
  const dateDisplay = dayjs(date).format('dddd, MMMM D');

  return (
    <IonPage>
      <IonHeader>
        <IonToolbar>
          <IonButtons slot="start">
            <IonBackButton defaultHref="/week" />
          </IonButtons>
          <IonTitle>{activity?.title ?? 'Activity'}</IonTitle>
        </IonToolbar>
      </IonHeader>
      <IonContent>
        {activitiesLoading ? (
          <div style={{ display: 'flex', justifyContent: 'center', padding: '40px' }}>
            <IonSpinner />
          </div>
        ) : !activity ? (
          <div style={{ padding: '24px', color: 'var(--fc-text-secondary)' }}>Activity not found.</div>
        ) : (
          <div style={s.body}>
            {isDone && <div style={s.doneChip}>DONE</div>}

            <div style={{ ...s.title, ...(isDone ? s.titleDone : {}) }}>{activity.title}</div>

            <div style={s.metaRow}>
              {activity.category && (
                <span style={s.badge}>{CATEGORY_LABELS[activity.category] ?? activity.category}</span>
              )}
              {activity.isTimeBound && (
                <span style={{ ...s.badge, color: 'var(--fc-accent)' }}>Time-bound</span>
              )}
              {isChore && (
                <span style={s.badge}>Repeating chore</span>
              )}
              {activity.startAt && !activity.isAllDay && (
                <span style={s.badge}>
                  {dayjs(activity.startAt).format('h:mm A')}
                  {activity.endAt ? ` – ${dayjs(activity.endAt).format('h:mm A')}` : ''}
                </span>
              )}
              {activity.isAllDay && <span style={s.badge}>All day</span>}
            </div>

            {activity.description && (
              <div style={s.desc}>{activity.description}</div>
            )}

            {isChore && (
              <>
                <div style={s.divider} />
                <div style={s.sectionLabel}>Completion</div>
                <div style={s.dateLabel}>{dateDisplay}</div>
                <button
                  style={isDone ? s.undoneBtn : s.doneBtn}
                  onClick={handleToggleDone}
                  disabled={isPending}
                >
                  {isPending
                    ? 'Saving…'
                    : isDone
                    ? 'Mark as not done'
                    : 'Mark as done'}
                </button>
              </>
            )}
          </div>
        )}
      </IonContent>
    </IonPage>
  );
};

export default ActivityDetail;
