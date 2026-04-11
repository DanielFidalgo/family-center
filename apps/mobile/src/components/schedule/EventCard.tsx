import React from 'react';
import type { MergedEventGroup, LocalActivity } from '@family-center/contracts';
import './EventCard.css';

export type EventCardItem =
  | { type: 'merged'; event: MergedEventGroup }
  | { type: 'activity'; activity: LocalActivity };

interface Props {
  item: EventCardItem;
  color?: string;
  compact?: boolean;
  onClick?: () => void;
}

function formatTime(iso: string): string {
  const d = new Date(iso);
  const h = d.getHours();
  const m = d.getMinutes();
  const ampm = h >= 12 ? 'pm' : 'am';
  const hour = h % 12 || 12;
  return m === 0 ? `${hour}${ampm}` : `${hour}:${m.toString().padStart(2, '0')}${ampm}`;
}

function getDurationMinutes(start: string, end: string): number {
  return (new Date(end).getTime() - new Date(start).getTime()) / 60000;
}

const EventCard: React.FC<Props> = ({ item, color = '#4A90D9', compact = false, onClick }) => {
  const title = item.type === 'merged' ? item.event.canonicalTitle : item.activity.title;
  const startAt = item.type === 'merged' ? item.event.canonicalStart : item.activity.startAt;
  const endAt = item.type === 'merged' ? item.event.canonicalEnd : item.activity.endAt;
  const isDupe = item.type === 'merged' && item.event.dupeTier;

  const duration = startAt && endAt ? getDurationMinutes(startAt, endAt) : 60;
  const height = Math.max(40, (duration / 60) * 80); // 80px per hour

  return (
    <div
      className={`event-card ${compact ? 'event-card--compact' : ''} ${isDupe ? 'event-card--dupe' : ''}`}
      style={{
        borderLeft: `4px solid ${color}`,
        minHeight: compact ? '32px' : `${height}px`,
        cursor: onClick ? 'pointer' : 'default',
      }}
      onClick={onClick}
      role={onClick ? 'button' : undefined}
    >
      <div className="event-card__title">{title}</div>
      {!compact && startAt && (
        <div className="event-card__time">
          {formatTime(startAt)}{endAt ? ` – ${formatTime(endAt)}` : ''}
        </div>
      )}
      {isDupe && <span className="event-card__dupe-badge">{item.event.dupeTier}</span>}
    </div>
  );
};

export default EventCard;
