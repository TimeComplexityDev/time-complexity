import { createContext, useContext, useState, useCallback, type ReactNode } from 'react';
import { v4 as uuid } from 'uuid';
import type { Watch, Evaluation, PositionReading } from '../types';
import { computeDefaultEvaluationName } from '../types';
import * as store from '../api/storage';

interface DataContextValue {
  watches: Watch[];
  evaluations: Evaluation[];
  positionReadings: PositionReading[];
  addWatch: (name: string, notes?: string) => Watch;
  updateWatch: (watch: Watch) => void;
  removeWatch: (id: string) => void;
  addEvaluation: (watchId: string, name?: string) => Evaluation;
  updateEvaluation: (evaluation: Evaluation) => void;
  removeEvaluation: (id: string) => void;
  addPositionReading: (evaluationId: string, position: PositionReading['position']) => PositionReading;
  updatePositionReading: (reading: PositionReading) => void;
  removePositionReading: (id: string) => void;
  refresh: () => void;
}

const DataContext = createContext<DataContextValue | null>(null);

export function DataProvider({ children }: { children: ReactNode }) {
  const [watches, setWatches] = useState<Watch[]>(store.getWatches);
  const [evaluations, setEvaluations] = useState<Evaluation[]>(store.getEvaluations);
  const [positionReadings, setPositionReadings] = useState<PositionReading[]>(store.getPositionReadings);

  const refresh = useCallback(() => {
    setWatches(store.getWatches());
    setEvaluations(store.getEvaluations());
    setPositionReadings(store.getPositionReadings());
  }, []);

  const addWatch = useCallback((name: string, notes = ''): Watch => {
    const watch: Watch = { id: uuid(), name, notes, created_at: new Date().toISOString() };
    store.saveWatch(watch);
    setWatches(store.getWatches());
    return watch;
  }, []);

  const updateWatch = useCallback((watch: Watch) => {
    store.saveWatch(watch);
    setWatches(store.getWatches());
  }, []);

  const removeWatch = useCallback((id: string) => {
    store.deleteWatch(id);
    setWatches(store.getWatches());
    setEvaluations(store.getEvaluations());
    setPositionReadings(store.getPositionReadings());
  }, []);

  const addEvaluation = useCallback((watchId: string, name?: string): Evaluation => {
    const watch = watches.find((w) => w.id === watchId);
    const evalName = name ?? (watch ? computeDefaultEvaluationName(watch.name) : '');
    const evaluation: Evaluation = {
      id: uuid(),
      watch_id: watchId,
      name: evalName,
      state: 'draft',
      bph: null,
      created_at: new Date().toISOString(),
    };
    store.saveEvaluation(evaluation);
    setEvaluations(store.getEvaluations());
    return evaluation;
  }, [watches]);

  const updateEvaluation = useCallback((evaluation: Evaluation) => {
    store.saveEvaluation(evaluation);
    setEvaluations(store.getEvaluations());
  }, []);

  const removeEvaluation = useCallback((id: string) => {
    store.deleteEvaluation(id);
    setEvaluations(store.getEvaluations());
    setPositionReadings(store.getPositionReadings());
  }, []);

  const addPositionReading = useCallback((evaluationId: string, position: PositionReading['position']): PositionReading => {
    const reading: PositionReading = {
      id: uuid(),
      evaluation_id: evaluationId,
      position,
      state: 'recording',
      session_id: null,
      rate_spd: null,
      beat_error_s: null,
      amplitude: null,
      completed_at: null,
    };
    store.savePositionReading(reading);
    setPositionReadings(store.getPositionReadings());
    return reading;
  }, []);

  const updatePositionReading = useCallback((reading: PositionReading) => {
    store.savePositionReading(reading);
    setPositionReadings(store.getPositionReadings());
  }, []);

  const removePositionReading = useCallback((id: string) => {
    store.deletePositionReading(id);
    setPositionReadings(store.getPositionReadings());
  }, []);

  return (
    <DataContext.Provider
      value={{
        watches,
        evaluations,
        positionReadings,
        addWatch,
        updateWatch,
        removeWatch,
        addEvaluation,
        updateEvaluation,
        removeEvaluation,
        addPositionReading,
        updatePositionReading,
        removePositionReading,
        refresh,
      }}
    >
      {children}
    </DataContext.Provider>
  );
}

export function useData(): DataContextValue {
  const ctx = useContext(DataContext);
  if (!ctx) throw new Error('useData must be used within DataProvider');
  return ctx;
}