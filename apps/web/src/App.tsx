import { useState, useCallback } from 'react';
import { PairingProvider } from './context/PairingContext';
import { DataProvider } from './context/DataContext';
import HomePage from './pages/HomePage';
import WatchPage from './pages/WatchPage';
import EvaluationPage from './pages/EvaluationPage';
import CapturePage from './pages/CapturePage';
import './App.css';

type Page =
  | { name: 'home' }
  | { name: 'watch'; watchId: string }
  | { name: 'evaluation'; evaluationId: string }
  | { name: 'capture'; readingId: string; evaluationId: string };

function AppContents() {
  const [page, setPage] = useState<Page>({ name: 'home' });

  const navigate = useCallback((name: string, params?: Record<string, string>) => {
    switch (name) {
      case 'home':
        setPage({ name: 'home' });
        break;
      case 'watch':
        setPage({ name: 'watch', watchId: params?.watchId ?? '' });
        break;
      case 'evaluation':
        setPage({ name: 'evaluation', evaluationId: params?.evaluationId ?? '' });
        break;
      case 'capture':
        setPage({ name: 'capture', readingId: params?.readingId ?? '', evaluationId: params?.evaluationId ?? '' });
        break;
    }
  }, []);

  switch (page.name) {
    case 'home':
      return <HomePage onNavigate={navigate} />;
    case 'watch':
      return <WatchPage watchId={page.watchId} onNavigate={navigate} />;
    case 'evaluation':
      return <EvaluationPage evaluationId={page.evaluationId} onNavigate={navigate} />;
    case 'capture':
      return <CapturePage readingId={page.readingId} evaluationId={page.evaluationId} onNavigate={navigate} />;
  }
}

export default function App() {
  return (
    <PairingProvider>
      <DataProvider>
        <AppContents />
      </DataProvider>
    </PairingProvider>
  );
}