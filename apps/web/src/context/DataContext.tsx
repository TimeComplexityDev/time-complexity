import { createContext, useContext, useState, useCallback, type ReactNode } from 'react';
import { v4 as uuid } from 'uuid';
import type { Watch, Evaluation, PositionReading } from '../types';
import { computeDefaultEvaluationName, allPositionsComplete } from '../types';
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
  transitionEvaluation: (id: string, target: 'in_progress' | 'complete') => void;
  markReadingComplete: (id: string, stats: { rate_spd: number; beat_error_s: number; amplitude: number }) => void;
  markReadingFailed: (id: string) => void;
  setEvaluationBph: (id: string, bph: number) => void;
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
    refresh();
    return watch;
  }, [refresh]);

  const updateWatch = useCallback((watch: Watch) => {
    store.saveWatch(watch);
    refresh();
  }, [refresh]);

  const removeWatch = useCallback((id: string) => {
    store.deleteWatch(id);
    refresh();
  }, [refresh]);

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
    refresh();
    return evaluation;
  }, [watches, refresh]);

  const updateEvaluation = useCallback((evaluation: Evaluation) => {
    store.saveEvaluation(evaluation);
    refresh();
  }, [refresh]);

  const removeEvaluation = useCallback((id: string) => {
    store.deleteEvaluation(id);
    refresh();
  }, [refresh]);

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
    refresh();
    return reading;
  }, [refresh]);

  const updatePositionReading = useCallback((reading: PositionReading) => {
    store.savePositionReading(reading);
    refresh();
  }, [refresh]);

  const removePositionReading = useCallback((id: string) => {
    store.deletePositionReading(id);
    refresh();
  }, [refresh]);

  const transitionEvaluation = useCallback((id: string, target: 'in_progress' | 'complete') => {
    const evals = store.getEvaluations();
    const evaluation = evals.find((e) => e.id === id);
    if (!evaluation) return;
    if (evaluation.state === 'complete') return;
    if (evaluation.state === 'draft' && target === 'in_progress') {
      evaluation.state = 'in_progress';
    } else if (evaluation.state === 'in_progress' && target === 'complete') {
      evaluation.state = 'complete';
    } else if (evaluation.state === 'draft' && target === 'complete') {
      evaluation.state = 'complete';
    }
    store.saveEvaluation(evaluation);
    refresh();
  }, [refresh]);

  const markReadingComplete = useCallback((id: string, stats: { rate_spd: number; beat_error_s: number; amplitude: number }) => {
    const readings = store.getPositionReadings();
    const reading = readings.find((r) => r.id === id);
    if (!reading) return;
    reading.state = 'complete';
    reading.rate_spd = stats.rate_spd;
    reading.beat_error_s = stats.beat_error_s;
    reading.amplitude = stats.amplitude;
    reading.completed_at = new Date().toISOString();
    store.savePositionReading(reading);

    // Check if all positions complete → auto-complete evaluation
    const allReadings = store.getPositionReadingsForEvaluation(reading.evaluation_id);
    const updated = allReadings.map((r) => (r.id === id ? reading : r));
    if (allPositionsComplete(updated)) {
      const evals = store.getEvaluations();
      const evaluation = evals.find((e) => e.id === reading.evaluation_id);
      if (evaluation && evaluation.state === 'in_progress') {
        evaluation.state = 'complete';
        store.saveEvaluation(evaluation);
      }
    }

    refresh();
  }, [refresh]);

  const markReadingFailed = useCallback((id: string) => {
    const readings = store.getPositionReadings();
    const reading = readings.find((r) => r.id === id);
    if (!reading) return;
    reading.state = 'failed';
    store.savePositionReading(reading);
    refresh();
  }, [refresh]);

  const setEvaluationBph = useCallback((id: string, bph: number) => {
    const evals = store.getEvaluations();
    const evaluation = evals.find((e) => e.id === id);
    if (!evaluation) return;
    evaluation.bph = bph;
    store.saveEvaluation(evaluation);
    refresh();
  }, [refresh]);

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
        transitionEvaluation,
        markReadingComplete,
        markReadingFailed,
        setEvaluationBph,
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