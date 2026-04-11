import React from 'react';
import { IonList, IonItem, IonLabel, IonRadioGroup, IonRadio } from '@ionic/react';
import type { DedupeMode } from '@family-center/contracts';

interface Props {
  value: DedupeMode;
  onChange: (mode: DedupeMode) => void;
}

const MODES: Array<{ value: DedupeMode; label: string; description: string }> = [
  { value: 'show_all', label: 'Show all', description: 'Show every event from every calendar' },
  { value: 'exact_only', label: 'Hide exact duplicates', description: 'Hide events with same iCal UID and time (default)' },
  { value: 'strong', label: 'Hide likely duplicates', description: 'Also hide events with same title and similar time' },
  { value: 'probable', label: 'Hide probable duplicates', description: 'Also hide events with same title/hour and overlapping attendees' },
];

const DedupeSettingsPanel: React.FC<Props> = ({ value, onChange }) => (
  <IonRadioGroup value={value} onIonChange={(e) => onChange(e.detail.value)}>
    <IonList>
      {MODES.map((mode) => (
        <IonItem key={mode.value}>
          <IonLabel>
            <h2>{mode.label}</h2>
            <p>{mode.description}</p>
          </IonLabel>
          <IonRadio slot="end" value={mode.value} />
        </IonItem>
      ))}
    </IonList>
  </IonRadioGroup>
);

export default DedupeSettingsPanel;
