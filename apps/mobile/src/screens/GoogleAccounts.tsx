import React from 'react';
import {
  IonPage, IonContent, IonHeader, IonToolbar, IonTitle,
  IonList, IonItem, IonLabel, IonButton, IonIcon, IonSpinner,
  IonNote,
} from '@ionic/react';
import { useHistory } from 'react-router-dom';
import { addOutline, calendarOutline } from 'ionicons/icons';
import { useGoogleAccounts, useConnectGoogleStart } from '../api/hooks';

const GoogleAccounts: React.FC = () => {
  const history = useHistory();
  const { data: accounts = [], isLoading, refetch } = useGoogleAccounts();
  const connectMutation = useConnectGoogleStart();

  const handleConnect = async () => {
    const res = await connectMutation.mutateAsync();
    if (res.authUrl.startsWith('/') || res.authUrl.startsWith('http://localhost')) {
      // Mock mode or local dev — treat as immediate callback
      await fetch(res.authUrl.replace('/google/connect/callback', `${import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000'}/google/connect/callback`))
        .catch(() => {});
      // In mock mode the callback creates the account immediately
      // Just call the callback endpoint directly
      try {
        const base = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000';
        await fetch(`${base}/google/connect/callback?code=mock&state=mock`);
      } catch {}
      refetch();
    } else {
      // Real OAuth — open browser
      window.open(res.authUrl, '_blank');
    }
  };

  return (
    <IonPage>
      <IonHeader>
        <IonToolbar>
          <IonTitle>Google Accounts</IonTitle>
        </IonToolbar>
      </IonHeader>
      <IonContent>
        {isLoading ? (
          <div style={{ display: 'flex', justifyContent: 'center', padding: '40px' }}>
            <IonSpinner />
          </div>
        ) : (
          <>
            <IonList>
              {accounts.map((account) => (
                <IonItem
                  key={account.id}
                  button
                  onClick={() => history.push(`/calendars/${account.id}`)}
                >
                  <IonIcon icon={calendarOutline} slot="start" />
                  <IonLabel>
                    <h2>{account.displayName ?? account.email}</h2>
                    <p>{account.email}</p>
                  </IonLabel>
                  <IonNote slot="end" color="medium">
                    Manage calendars
                  </IonNote>
                </IonItem>
              ))}
            </IonList>

            <div style={{ padding: '20px' }}>
              <IonButton
                expand="block"
                onClick={handleConnect}
                disabled={connectMutation.isPending}
              >
                {connectMutation.isPending ? <IonSpinner name="crescent" /> : (
                  <>
                    <IonIcon icon={addOutline} slot="start" />
                    Connect Google Account
                  </>
                )}
              </IonButton>
              <p style={{ fontSize: '12px', color: '#a0a0b0', textAlign: 'center', marginTop: '12px' }}>
                Only calendar read access is requested. Tokens are stored on your server.
              </p>
            </div>
          </>
        )}
      </IonContent>
    </IonPage>
  );
};

export default GoogleAccounts;
