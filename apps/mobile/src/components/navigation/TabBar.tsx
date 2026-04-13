import React from 'react';
import { useIonRouter } from '@ionic/react';
import { useHistory } from 'react-router-dom';

const TABS = [
  {
    path: '/day',
    label: 'Today',
    icon: (
      <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <rect x="3" y="4" width="18" height="18" rx="2" ry="2"/>
        <line x1="16" y1="2" x2="16" y2="6"/>
        <line x1="8" y1="2" x2="8" y2="6"/>
        <line x1="3" y1="10" x2="21" y2="10"/>
        <circle cx="12" cy="16" r="1.5" fill="currentColor" stroke="none"/>
      </svg>
    ),
  },
  {
    path: '/week',
    label: 'Week',
    icon: (
      <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <rect x="3" y="4" width="18" height="18" rx="2" ry="2"/>
        <line x1="16" y1="2" x2="16" y2="6"/>
        <line x1="8" y1="2" x2="8" y2="6"/>
        <line x1="3" y1="10" x2="21" y2="10"/>
        <line x1="3" y1="15" x2="21" y2="15"/>
        <line x1="8" y1="10" x2="8" y2="22"/>
        <line x1="16" y1="10" x2="16" y2="22"/>
      </svg>
    ),
  },
  {
    path: '/people',
    label: 'People',
    icon: (
      <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
        <circle cx="9" cy="7" r="4"/>
        <path d="M23 21v-2a4 4 0 0 0-3-3.87"/>
        <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
      </svg>
    ),
  },
  {
    path: '/settings',
    label: 'Settings',
    icon: (
      <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <circle cx="12" cy="12" r="3"/>
        <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
      </svg>
    ),
  },
];

const TabBar: React.FC = () => {
  const router = useIonRouter();
  const history = useHistory();
  const currentPath = router.routeInfo.pathname;

  return (
    <nav className="fc-tabbar">
      {TABS.map(({ path, label, icon }) => {
        const active = currentPath === path;
        return (
          <button
            key={path}
            className={`fc-tabbar__btn${active ? ' fc-tabbar__btn--active' : ''}`}
            onClick={() => history.push(path)}
          >
            <span className="fc-tabbar__icon">{icon}</span>
            <span className="fc-tabbar__label">{label}</span>
            {active && <span className="fc-tabbar__indicator" />}
          </button>
        );
      })}

      <style>{`
        .fc-tabbar {
          display: flex;
          flex-direction: row;
          align-items: stretch;
          height: var(--fc-tab-height, 64px);
          background: var(--fc-bg-surface);
          border-top: 1px solid var(--fc-border);
          flex-shrink: 0;
          position: relative;
        }

        .fc-tabbar__btn {
          flex: 1;
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          gap: 4px;
          background: none;
          border: none;
          cursor: pointer;
          color: var(--fc-text-secondary);
          padding: 8px 4px;
          position: relative;
          transition: color 0.18s ease;
          -webkit-tap-highlight-color: transparent;
          outline: none;
        }

        .fc-tabbar__btn--active {
          color: var(--fc-accent);
        }

        .fc-tabbar__icon {
          display: flex;
          align-items: center;
          justify-content: center;
          line-height: 1;
        }

        .fc-tabbar__label {
          font-family: var(--fc-font-body);
          font-size: 11px;
          font-weight: 600;
          letter-spacing: 0.02em;
          text-transform: uppercase;
          line-height: 1;
        }

        .fc-tabbar__indicator {
          position: absolute;
          top: 0;
          left: 50%;
          transform: translateX(-50%);
          width: 32px;
          height: 2px;
          background: var(--fc-accent);
          border-radius: 0 0 3px 3px;
        }
      `}</style>
    </nav>
  );
};

export default TabBar;
