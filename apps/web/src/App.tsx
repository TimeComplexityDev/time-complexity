import { useState } from 'react';
import { PairingProvider } from './context/PairingContext';
import { DataProvider } from './context/DataContext';
import HomePage from './pages/HomePage';
import WatchPage from './pages/WatchPage';
import EvaluationPage from './pages/EvaluationPage';
import CapturePage from './pages/CapturePage';
import type { Page } from './types';
import './App.css';

function AppContents() {
  const [page, setPage] = useState<Page>({ name: 'home' });

  switch (page.name) {
    case 'home':
      return <HomePage onNavigate={setPage} />;
    case 'watch':
      return <WatchPage watchId={page.watchId} onNavigate={setPage} />;
    case 'evaluation':
      return <EvaluationPage evaluationId={page.evaluationId} onNavigate={setPage} />;
    case 'capture':
      return <CapturePage readingId={page.readingId} evaluationId={page.evaluationId} onNavigate={setPage} />;
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