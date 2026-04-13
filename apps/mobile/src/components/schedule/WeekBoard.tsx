import React from 'react';
import type { Person, MergedEventGroup, LocalActivity, ActivityCompletion, Settings } from '@family-center/contracts';
import Avatar from '../Avatar';
import './WeekBoard.css';

interface Props {
  people: Person[];
  events: MergedEventGroup[];
  activities: LocalActivity[];
  completions?: ActivityCompletion[];
  weekStart: Date;
  settings?: Settings;
  onEventClick?: (event: MergedEventGroup) => void;
  onActivityClick?: (activity: LocalActivity, date: Date) => void;
  onDayClick?: (date: Date) => void;
}

function addDays(date: Date, days: number): Date {
  const d = new Date(date);
  d.setDate(d.getDate() + days);
  return d;
}

const DAY_SHORT = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];

function formatTime(iso: string): string {
  const d = new Date(iso);
  const h = d.getHours();
  const m = d.getMinutes();
  const ampm = h >= 12 ? 'pm' : 'am';
  const hour = h % 12 || 12;
  return m === 0 ? `${hour}${ampm}` : `${hour}:${m.toString().padStart(2, '0')}${ampm}`;
}

interface LaneCell {
  events: MergedEventGroup[];
  activities: LocalActivity[];
}

const WeekBoard: React.FC<Props> = ({
  people,
  events,
  activities,
  completions = [],
  weekStart,
  settings,
  onEventClick,
  onActivityClick,
  onDayClick,
}) => {
  const isActivityDone = (activityId: string, dayDate: Date): boolean => {
    const dateStr = dayDate.toISOString().slice(0, 10);
    return completions.some((c) => c.localActivityId === activityId && c.completedDate === dateStr);
  };
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const dedupeMode = settings?.dedupeMode ?? 'exact_only';

  const days = Array.from({ length: 7 }, (_, i) => addDays(weekStart, i));

  const lanes = [
    { id: null as string | null, name: 'Shared', color: 'var(--fc-shared-lane-color)', avatarUrl: undefined as string | undefined },
    ...people.map((p) => ({ id: p.id, name: p.name, color: p.color, avatarUrl: p.avatarUrl })),
  ];

  const getCell = (dayDate: Date, laneId: string | null): LaneCell => {
    const start = new Date(dayDate);
    start.setHours(0, 0, 0, 0);
    const end = new Date(dayDate);
    end.setHours(23, 59, 59, 999);

    const dayEvents = events.filter((e) => {
      const t = new Date(e.canonicalStart).getTime();
      return t >= start.getTime() && t <= end.getTime() &&
        (laneId ? e.personId === laneId : !e.personId);
    });

    const dayActivities = activities.filter((a) => {
      if (!a.startAt) return false;
      const t = new Date(a.startAt).getTime();
      return t >= start.getTime() && t <= end.getTime() &&
        (laneId ? a.personId === laneId : !a.personId);
    });

    // Apply dedupe filtering
    const visibleEvents = dayEvents.filter((e) => {
      if (dedupeMode === 'show_all') return true;
      if (dedupeMode === 'exact_only') return e.dupeTier !== 'exact' || e.sources?.[0]?.isPrimary;
      if (dedupeMode === 'strong') return !e.dupeTier || e.dupeTier === 'probable';
      if (dedupeMode === 'probable') return !e.dupeTier;
      return true;
    });

    return { events: visibleEvents, activities: dayActivities };
  };

  return (
    <div className="wsw">
      {/* Sticky header row: lane labels across the top */}
      <div className="wsw__head">
        <div className="wsw__corner" />
        {lanes.map((lane) => (
          <div
            key={lane.id ?? 'shared'}
            className="wsw__lane-head"
            style={{ borderTop: `3px solid ${lane.color}` }}
          >
            <Avatar
              name={lane.name}
              color={lane.color}
              avatarUrl={lane.avatarUrl}
              size={24}
            />
            <span className="wsw__lane-name">{lane.name}</span>
          </div>
        ))}
      </div>

      {/* Day rows */}
      <div className="wsw__body">
        {days.map((day, di) => {
          const isToday = day.getTime() === today.getTime();
          const isSat = di === 5;
          const isSun = di === 6;

          return (
            <div
              key={di}
              className={[
                'wsw__row',
                isToday ? 'wsw__row--today' : '',
                (isSat || isSun) ? 'wsw__row--weekend' : '',
              ].join(' ')}
            >
              {/* Day label */}
              <button
                className={`wsw__day-label${isToday ? ' wsw__day-label--today' : ''}`}
                onClick={onDayClick ? () => onDayClick(day) : undefined}
              >
                <span className="wsw__day-name">{DAY_SHORT[di]}</span>
                <span className={`wsw__day-num${isToday ? ' wsw__day-num--today' : ''}`}>
                  {day.getDate()}
                </span>
              </button>

              {/* Lane cells */}
              {lanes.map((lane) => {
                const cell = getCell(day, lane.id);
                const allItems = [
                  ...cell.events.map((e) => ({ type: 'event' as const, id: e.id, title: e.canonicalTitle, startAt: e.canonicalStart, data: e })),
                  ...cell.activities.map((a) => ({ type: 'activity' as const, id: a.id, title: a.title, startAt: a.startAt ?? '', data: a, color: a.color })),
                ].sort((a, b) => new Date(a.startAt).getTime() - new Date(b.startAt).getTime());

                return (
                  <div
                    key={lane.id ?? 'shared'}
                    className={`wsw__cell${allItems.length === 0 ? ' wsw__cell--empty' : ''}`}
                    style={{ '--lane-color': lane.color } as React.CSSProperties}
                  >
                    {allItems.map((item) => {
                      const done = item.type === 'activity' && isActivityDone(item.id, day);
                      return (
                        <button
                          key={item.id}
                          className={[
                            'wsw__chip',
                            item.type === 'activity' ? 'wsw__chip--activity' : '',
                            done ? 'wsw__chip--done' : '',
                          ].filter(Boolean).join(' ')}
                          style={{ borderLeftColor: item.type === 'activity' && item.color ? item.color : lane.color }}
                          onClick={
                            item.type === 'event' && onEventClick
                              ? () => onEventClick(item.data as MergedEventGroup)
                              : item.type === 'activity' && onActivityClick
                              ? () => onActivityClick(item.data as LocalActivity, day)
                              : undefined
                          }
                        >
                          {item.startAt && (
                            <span className="wsw__chip-time">{formatTime(item.startAt)}</span>
                          )}
                          <span className={`wsw__chip-title${done ? ' wsw__chip-title--done' : ''}`}>{item.title}</span>
                          {done
                            ? <span className="wsw__chip-done-label">DONE</span>
                            : item.type === 'activity' && <span className="wsw__chip-badge">✓</span>
                          }
                        </button>
                      );
                    })}
                  </div>
                );
              })}
            </div>
          );
        })}
      </div>
    </div>
  );
};

export default WeekBoard;
