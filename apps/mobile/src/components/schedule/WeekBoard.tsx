import React from 'react';
import type { Person, MergedEventGroup, LocalActivity, Settings } from '@family-center/contracts';
import LaneBoard from './LaneBoard';
import './WeekBoard.css';

interface Props {
  people: Person[];
  events: MergedEventGroup[];
  activities: LocalActivity[];
  weekStart: Date;
  settings?: Settings;
  onEventClick?: (event: MergedEventGroup) => void;
  onDayClick?: (date: Date) => void;
}

function addDays(date: Date, days: number): Date {
  const d = new Date(date);
  d.setDate(d.getDate() + days);
  return d;
}

const DAYS = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];

const WeekBoard: React.FC<Props> = ({
  people,
  events,
  activities,
  weekStart,
  settings,
  onEventClick,
  onDayClick,
}) => {
  const today = new Date();
  today.setHours(0, 0, 0, 0);

  const days = Array.from({ length: 7 }, (_, i) => addDays(weekStart, i));

  const eventsForDay = (date: Date): MergedEventGroup[] => {
    const start = new Date(date);
    start.setHours(0, 0, 0, 0);
    const end = new Date(date);
    end.setHours(23, 59, 59, 999);
    return events.filter((e) => {
      const t = new Date(e.canonicalStart).getTime();
      return t >= start.getTime() && t <= end.getTime();
    });
  };

  const activitiesForDay = (date: Date): LocalActivity[] => {
    const start = new Date(date);
    start.setHours(0, 0, 0, 0);
    const end = new Date(date);
    end.setHours(23, 59, 59, 999);
    return activities.filter((a) => {
      if (!a.startAt) return false;
      const t = new Date(a.startAt).getTime();
      return t >= start.getTime() && t <= end.getTime();
    });
  };

  return (
    <div className="week-board">
      {days.map((day, i) => {
        const isToday = day.getTime() === today.getTime();
        const dayEvents = eventsForDay(day);
        const dayActivities = activitiesForDay(day);
        const totalItems = dayEvents.length + dayActivities.length;

        return (
          <div
            key={i}
            className={`week-board__day ${isToday ? 'week-board__day--today' : ''}`}
            onClick={onDayClick ? () => onDayClick(day) : undefined}
          >
            <div className={`week-board__day-header ${isToday ? 'week-board__day-header--today' : ''}`}>
              <div className="week-board__day-name">{DAYS[i]}</div>
              <div className="week-board__day-num">{day.getDate()}</div>
              {totalItems > 0 && (
                <div className="week-board__day-count">{totalItems}</div>
              )}
            </div>
            <div className="week-board__day-lanes">
              <LaneBoard
                people={people}
                events={dayEvents}
                activities={dayActivities}
                date={day}
                settings={settings}
                onEventClick={onEventClick}
              />
            </div>
          </div>
        );
      })}
    </div>
  );
};

export default WeekBoard;
