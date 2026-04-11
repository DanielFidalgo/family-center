import React, { useState } from 'react';
import {
  IonPage, IonContent, IonButton, IonInput, IonItem,
  IonLabel, IonSpinner, IonText,
} from '@ionic/react';
import { api } from '../api/client';
import { useAppStore } from '../store/appStore';
import type { BootstrapResponse } from '@family-center/contracts';

const Onboarding: React.FC = () => {
  const [householdName, setHouseholdName] = useState('My Family');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const setHousehold = useAppStore((s) => s.setHousehold);

  const handleSetup = async () => {
    setLoading(true);
    setError('');
    try {
      const res = await api.post<BootstrapResponse>('/auth/bootstrap', {
        householdName,
      });
      setHousehold(res.householdId, res.token);
    } catch (e: any) {
      setError(e.message ?? 'Setup failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <IonPage>
      <IonContent>
        <div style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          height: '100%',
          padding: '40px',
          gap: '24px',
        }}>
          <div style={{ textAlign: 'center' }}>
            <h1 style={{ fontSize: '32px', fontWeight: 700, color: '#e0e0e0', margin: 0 }}>
              Family Center
            </h1>
            <p style={{ color: '#a0a0b0', marginTop: '8px' }}>
              Your wall-mounted family scheduler
            </p>
          </div>

          <div style={{ width: '100%', maxWidth: '400px' }}>
            <IonItem lines="full" style={{ '--background': '#16213e', borderRadius: '8px' }}>
              <IonLabel position="stacked" style={{ color: '#a0a0b0' }}>
                Family Name
              </IonLabel>
              <IonInput
                value={householdName}
                onIonChange={(e) => setHouseholdName(e.detail.value ?? '')}
                placeholder="e.g. The Smiths"
                style={{ '--color': '#e0e0e0' }}
              />
            </IonItem>
          </div>

          {error && (
            <IonText color="danger">
              <p>{error}</p>
            </IonText>
          )}

          <IonButton
            expand="block"
            style={{ width: '100%', maxWidth: '400px', '--border-radius': '8px', minHeight: '52px' }}
            onClick={handleSetup}
            disabled={loading || !householdName.trim()}
          >
            {loading ? <IonSpinner name="crescent" /> : 'Get Started'}
          </IonButton>
        </div>
      </IonContent>
    </IonPage>
  );
};

export default Onboarding;
