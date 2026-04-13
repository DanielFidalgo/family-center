import React, { useRef, useEffect } from 'react';
import type { Person, MergedEventGroup, LocalActivity, ActivityCompletion, Settings } from '@family-center/contracts';
import Avatar from '../Avatar';
import './DayTimeline.css';

interface Props {
  people: Person[];
  events: MergedEventGroup[];
  activities: LocalActivity[];
  completions?: ActivityCompletion[];
  date: Date;
  settings?: Settings;
  onEventClick?: (event: MergedEventGroup) => void;
  onActivityClick?: (activity: LocalActivity) => void;
}

const HOURS = Array.from({ length: 24 }, (_, i) => i);
const HOUR_HEIGHT = 80;

function eventPosition(start: string, end: string, baseDate: Date) {
  const startDate = new Date(start);
  const startOfDay = new Date(baseDate);
  startOfDay.setHours(0, 0, 0, 0);
  const offsetHours = (startDate.getTime() - startOfDay.getTime()) / 3600000;
  if (offsetHours < 0 || offsetHours > 24) return null;
  const durationHours = (new Date(end).getTime() - startDate.getTime()) / 3600000;
  return {
    top: offsetHours * HOUR_HEIGHT,
    height: Math.max(32, durationHours * HOUR_HEIGHT),
  };
}

function getNowOffsetPx(date: Date): number | null {
  const now = new Date();
  const today = new Date(date);
  today.setHours(0, 0, 0, 0);
  if (now.toDateString() !== date.toDateString()) return null;
  const offsetHours = (now.getTime() - today.getTime()) / 3600000;
  return offsetHours * HOUR_HEIGHT;
}

const DayTimeline: React.FC<Props> = ({
  people,
  events,
  activities,
  completions = [],
  date,
  settings,
  onEventClick,
  onActivityClick,
}) => {
  const dateStr = date.toISOString().slice(0, 10);
  const isActivityDone = (activityId: string) =>
    completions.some((c) => c.localActivityId === activityId && c.completedDate === dateStr);
  const scrollRef = useRef<HTMLDivElement>(null);

  // Scroll to current time on mount
  useEffect(() => {
    const nowPx = getNowOffsetPx(date);
    if (nowPx !== null && scrollRef.current) {
      const scrollTo = Math.max(0, nowPx - 120);
      scrollRef.current.scrollTop = scrollTo;
    }
  }, [date]);

  const dedupeMode = settings?.dedupeMode ?? 'exact_only';

  const eventsForPerson = (personId: string | null) => {
    const raw = events.filter((e) => (personId ? e.personId === personId : !e.personId));
    return raw.filter((e) => {
      if (dedupeMode === 'show_all') return true;
      if (dedupeMode === 'exact_only') return e.dupeTier !== 'exact' || e.sources?.[0]?.isPrimary;
      if (dedupeMode === 'strong') return !e.dupeTier || e.dupeTier === 'probable';
      if (dedupeMode === 'probable') return !e.dupeTier;
      return true;
    });
  };

  const activitiesForPerson = (personId: string | null) =>
    activities.filter((a) => (personId ? a.personId === personId : !a.personId));

  const lanes = [
    { person: null as Person | null, label: 'Shared', color: 'var(--fc-shared-lane-color)' },
    ...people.map((p) => ({ person: p, label: p.name, color: p.color })),
  ];

  const nowPx = getNowOffsetPx(date);

  return (
    <div className="day-timeline">
      {/* Column headers */}
      <div className="day-timeline__headers">
        <div className="day-timeline__time-gutter" />
        {lanes.map(({ person, label, color }) => (
          <div
            key={person?.id ?? 'shared'}
            className="day-timeline__col-header"
            style={{ borderTop: `3px solid ${color}` }}
          >
            <Avatar
              name={label}
              color={color}
              avatarUrl={person?.avatarUrl}
              size={24}
            />
            <span className="day-timeline__col-label">{label}</span>
          </div>
        ))}
      </div>

      {/* Scrollable time grid */}
      <div className="day-timeline__scroll" ref={scrollRef}>
        <div className="day-timeline__grid" style={{ height: `${HOUR_HEIGHT * 24}px` }}>
          {/* Time labels */}
          <div className="day-timeline__time-col">
            {HOURS.map((h) => (
              <div
                key={h}
                className="day-timeline__time-label"
                style={{ top: `${h * HOUR_HEIGHT}px` }}
              >
                {h === 0 ? '12am' : h < 12 ? `${h}am` : h === 12 ? '12pm' : `${h - 12}pm`}
              </div>
            ))}
          </div>

          {/* Lane columns */}
          {lanes.map(({ person, color }) => {
            const pid = person?.id ?? null;
            const laneEvents = eventsForPerson(pid);
            const laneActivities = activitiesForPerson(pid);

            return (
              <div key={pid ?? 'shared'} className="day-timeline__lane-col">
                {/* Hour grid lines */}
                {HOURS.map((h) => (
                  <div
                    key={h}
                    className="day-timeline__hour-line"
                    style={{ top: `${h * HOUR_HEIGHT}px` }}
                  />
                ))}

                {/* Current time indicator */}
                {nowPx !== null && (
                  <div
                    className="day-timeline__now-line"
                    style={{ top: `${nowPx}px` }}
                  />
                )}

                {/* Calendar events */}
                {laneEvents.map((ev) => {
                  const pos = eventPosition(ev.canonicalStart, ev.canonicalEnd, date);
                  if (!pos) return null;
                  return (
                    <div
                      key={ev.id}
                      className="day-timeline__event"
                      role={onEventClick ? 'button' : undefined}
                      style={{
                        top: `${pos.top}px`,
                        height: `${pos.height}px`,
                        borderLeft: `3px solid ${color}`,
                      }}
                      onClick={onEventClick ? () => onEventClick(ev) : undefined}
                    >
                      <div className="day-timeline__event-title">{ev.canonicalTitle}</div>
                      <div className="day-timeline__event-time">
                        {new Date(ev.canonicalStart).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                      </div>
                    </div>
                  );
                })}

                {/* Local activities */}
                {laneActivities.filter((a) => a.startAt).map((act) => {
                  const pos = eventPosition(act.startAt!, act.endAt ?? act.startAt!, date);
                  if (!pos) return null;
                  const done = isActivityDone(act.id);
                  return (
                    <div
                      key={act.id}
                      className={`day-timeline__event day-timeline__event--activity${done ? ' day-timeline__event--done' : ''}`}
                      role={onActivityClick ? 'button' : undefined}
                      style={{
                        top: `${pos.top}px`,
                        height: `${pos.height}px`,
                        borderLeft: `3px solid ${act.color ?? color}`,
                      }}
                      onClick={onActivityClick ? () => onActivityClick(act) : undefined}
                    >
                      <div className={`day-timeline__event-title${done ? ' day-timeline__event-title--done' : ''}`}>{act.title}</div>
                      {done
                        ? <span className="day-timeline__event-done-label">DONE</span>
                        : <span className="day-timeline__event-badge">✓</span>
                      }
                    </div>
                  );
                })}
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};

export default DayTimeline;
