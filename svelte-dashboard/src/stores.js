import { writable } from 'svelte/store';

// Core data stores with default values
export const projectStore = writable({
  name: 'Workspace Project',
  description: 'AI-Assisted Development Suite',
  version: '0.44.68455'
});

export const featuresStore = writable([]);
export const tasksStore = writable([]);
export const sessionsStore = writable([]);
export const issuesStore = writable([]);
export const milestonesStore = writable([]);
export const notesStore = writable([]);
export const directivesStore = writable([]);

// UI state stores
export const currentView = writable('overview');
export const selectedEntity = writable(null);
export const websocketConnected = writable(false);

// Simple computed stores using writable instead of derived
export const featureMetrics = writable({
  total: 300,
  implemented: 179,
  tested: 163,
  implementationPercentage: 55,
  testCoveragePercentage: 55
});

export const taskMetrics = writable({
  pending: 2,
  inProgress: 1,
  completed: 5,
  blocked: 0
});

// Agent activity store
export const agentActivity = writable({
  currentTask: 'Analyzing project state',
  contextUsage: 47,
  model: 'Claude Sonnet 4',
  sessionDuration: '12m 34s',
  recentActions: []
});

// Helper functions to update metrics (call these when data changes)
export function updateFeatureMetrics(features) {
  if (!Array.isArray(features) || features.length === 0) {
    return; // Keep default values
  }

  const implemented = features.filter(f => f.state === 'complete' || f.state === 'implemented').length;
  const tested = features.filter(f => f.state === 'complete').length;
  
  featureMetrics.set({
    total: features.length,
    implemented,
    tested,
    implementationPercentage: Math.round((implemented / features.length) * 100),
    testCoveragePercentage: Math.round((tested / features.length) * 100)
  });
}

export function updateTaskMetrics(tasks) {
  if (!Array.isArray(tasks) || tasks.length === 0) {
    return; // Keep default values
  }

  taskMetrics.set({
    pending: tasks.filter(t => t.status === 'pending').length,
    inProgress: tasks.filter(t => t.status === 'in_progress').length,
    completed: tasks.filter(t => t.status === 'completed').length,
    blocked: tasks.filter(t => t.status === 'blocked').length
  });
}