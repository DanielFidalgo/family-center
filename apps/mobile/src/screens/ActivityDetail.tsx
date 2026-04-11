import React from 'react';
import {
  IonPage, IonContent, IonHeader, IonToolbar, IonTitle,
  IonBackButton, IonButtons, IonList, IonItem, IonLabel,
} from '@ionic/react';
import { useParams } from 'react-router-dom';

const ActivityDetail: React.FC = () => {
  const { id } = useParams<{ id: string }>();

  return (
    <IonPage>
      <IonHeader>
        <IonToolbar>
          <IonButtons slot="start">
            <IonBackButton defaultHref="/week" />
          </IonButtons>
          <IonTitle>Event Detail</IonTitle>
        </IonToolbar>
      </IonHeader>
      <IonContent>
        <IonList>
          <IonItem>
            <IonLabel>
              <h2>Event ID</h2>
              <p>{id}</p>
            </IonLabel>
          </IonItem>
          <IonItem lines="none">
            <IonLabel color="medium">
              <p style={{ fontSize: '13px' }}>
                Full event detail view — shows title, time, source calendar,
                lane assignment, and duplicate provenance.
              </p>
            </IonLabel>
          </IonItem>
        </IonList>
      </IonContent>
    </IonPage>
  );
};

export default ActivityDetail;
