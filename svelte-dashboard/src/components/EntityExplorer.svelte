<script>
  import { onMount } from 'svelte';
  import { featuresStore, tasksStore, notesStore } from '../stores.js';
  
  let selectedEntityType = 'all';
  let entityCounts = {};
  
  const entityTypes = [
    { id: 'all', label: 'All Entities', icon: 'A', color: '#9b59d0' },
    { id: 'features', label: 'Features', icon: 'F', color: '#f59e0b' },
    { id: 'tasks', label: 'Tasks', icon: 'T', color: '#4ade80' },
    { id: 'sessions', label: 'Sessions', icon: 'S', color: '#06b6d4' },
    { id: 'notes', label: 'Notes', icon: 'N', color: '#8b5cf6' },
    { id: 'milestones', label: 'Milestones', icon: 'M', color: '#ef4444' }
  ];

  $: {
    // Calculate entity counts
    entityCounts = {
      features: $featuresStore.length || 300,
      tasks: $tasksStore.length || 25,
      sessions: 8, // Placeholder
      notes: $notesStore.length || 42,
      milestones: 6, // Placeholder
      total: 381 // Total placeholder
    };
  }

  function selectEntityType(type) {
    selectedEntityType = type;
  }
</script>

<div class="entity-explorer-card card bg-surface">
  <h2 class="text-primary">Entity Explorer</h2>
  
  <div class="entity-filters">
    {#each entityTypes as entity}
      <button 
        class="entity-filter btn-secondary entity-item-{entity.id}" 
        class:active={selectedEntityType === entity.id}
        on:click={() => selectEntityType(entity.id)}
      >
        <span class="entity-icon">{entity.icon}</span>
        <span class="entity-label text-secondary">{entity.label}</span>
        {#if entity.id !== 'all'}
          <span class="entity-count text-primary">{entityCounts[entity.id] || 0}</span>
        {/if}
      </button>
    {/each}
  </div>

  <div class="entity-stats">
    <div class="total-entities">
      <div class="total-count text-primary">{entityCounts.total}</div>
      <div class="total-label text-secondary">Total Entities</div>
    </div>
    
    <div class="entity-breakdown">
      {#each entityTypes.slice(1) as entity}
        <div class="breakdown-item entity-item-{entity.id}">
          <div class="breakdown-bar">
            <div class="breakdown-fill" style:width="{((entityCounts[entity.id] || 0) / entityCounts.total * 100)}%"></div>
          </div>
          <div class="breakdown-info">
            <span class="breakdown-label text-secondary">{entity.label}</span>
            <span class="breakdown-count text-primary">{entityCounts[entity.id] || 0}</span>
          </div>
        </div>
      {/each}
    </div>
  </div>

  <div class="relationship-summary">
    <h3 class="text-primary">Entity Relationships</h3>
    <div class="relationship-stats">
      <div class="relationship-item">
        <span class="relationship-label text-secondary">Dependencies</span>
        <span class="relationship-count text-primary">156</span>
      </div>
      <div class="relationship-item">
        <span class="relationship-label text-secondary">References</span>
        <span class="relationship-count text-primary">89</span>
      </div>
      <div class="relationship-item">
        <span class="relationship-label text-secondary">Blocks</span>
        <span class="relationship-count text-primary">12</span>
      </div>
    </div>
    
    <div class="network-preview">
      <div class="network-node features" data-position="20,20"></div>
      <div class="network-node tasks" data-position="40,60"></div>
      <div class="network-node sessions" data-position="70,30"></div>
      <div class="network-node notes" data-position="60,80"></div>
      <div class="network-connection connection-1" data-connection="25,25,35,15"></div>
      <div class="network-connection connection-2" data-connection="45,40,25,-20"></div>
      <div class="network-connection connection-3" data-connection="65,50,30,35"></div>
    </div>
  </div>
</div>

<style>
  .entity-explorer-card {
    border-radius: 0.75rem;
    padding: 1.5rem;
  }

  .entity-explorer-card h2 {
    margin: 0 0 1.25rem 0;
    font-size: 1.25rem;
    font-weight: 600;
  }

  .entity-filters {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-bottom: 1.25rem;
  }

  .entity-filter {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.5rem 0.75rem;
    border-radius: 1.25rem;
    font-size: 0.75rem;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .entity-filter:hover {
    background-color: var(--hover-bg);
  }

  .entity-filter.active {
    background-color: var(--accent-bg);
    border-color: var(--accent-color);
  }

  .entity-icon {
    font-size: 0.875rem;
  }

  .entity-count {
    padding: 0.125rem 0.375rem;
    border-radius: 0.625rem;
    font-size: 0.625rem;
    font-weight: 600;
    background-color: var(--bg-surface-3);
  }

  .entity-stats {
    margin-bottom: 1.5rem;
  }

  .total-entities {
    text-align: center;
    margin-bottom: 1rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid var(--border-color);
  }

  .total-count {
    font-size: 2rem;
    font-weight: 700;
    line-height: 1;
  }

  .total-label {
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .entity-breakdown {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .breakdown-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .breakdown-bar {
    flex: 1;
    height: 4px;
    border-radius: 2px;
    overflow: hidden;
    background-color: var(--bg-surface-3);
  }

  .breakdown-fill {
    height: 100%;
    transition: width 0.6s ease;
    background-color: var(--accent-color);
  }

  .breakdown-info {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    min-width: 5rem;
  }

  .breakdown-label {
    font-size: 0.6875rem;
  }

  .breakdown-count {
    font-size: 0.6875rem;
    font-weight: 600;
    font-family: var(--font-mono);
  }

  .relationship-summary {
    border-top: 1px solid var(--border-color);
    padding-top: 1.25rem;
  }

  .relationship-summary h3 {
    font-size: 0.875rem;
    font-weight: 600;
    margin: 0 0 1rem 0;
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .relationship-stats {
    display: flex;
    justify-content: space-between;
    margin-bottom: 1rem;
  }

  .relationship-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    font-size: 0.75rem;
  }

  .relationship-count {
    font-weight: 600;
    font-size: 0.875rem;
  }

  .network-preview {
    position: relative;
    height: 6.25rem;
    border-radius: 0.5rem;
    overflow: hidden;
    background-color: var(--bg-surface-2);
  }

  .network-node {
    position: absolute;
    width: 0.5rem;
    height: 0.5rem;
    border-radius: 50%;
    animation: pulse 2s infinite;
    background-color: var(--accent-color);
  }

  .network-node.features {
    background-color: var(--color-warning);
    top: 20%; left: 20%;
  }

  .network-node.tasks {
    background-color: var(--color-success);
    top: 40%; left: 60%;
  }

  .network-node.sessions {
    background-color: var(--color-info);
    top: 70%; left: 30%;
  }

  .network-node.notes {
    background-color: var(--color-accent);
    top: 60%; left: 80%;
  }

  .network-connection {
    position: absolute;
    height: 1px;
    transform-origin: left center;
    background-color: var(--color-border);
  }

  .connection-1 {
    top: 25%; left: 25%; width: 35%; transform: rotate(15deg);
  }

  .connection-2 {
    top: 45%; left: 40%; width: 25%; transform: rotate(-20deg);
  }

  .connection-3 {
    top: 65%; left: 50%; width: 30%; transform: rotate(35deg);
  }

  @keyframes pulse {
    0%, 100% { opacity: 0.6; transform: scale(1); }
    50% { opacity: 1; transform: scale(1.2); }
  }

  /* Entity-specific colors */
  .entity-item-features { --entity-color: var(--color-warning); }
  .entity-item-tasks { --entity-color: var(--color-success); }
  .entity-item-sessions { --entity-color: var(--color-info); }
  .entity-item-notes { --entity-color: var(--color-accent); }
  .entity-item-goals { --entity-color: var(--color-error); }
  .entity-item-contexts { --entity-color: var(--color-info); }
  .entity-item-projects { --entity-color: var(--color-warning); }
  .entity-item-issues { --entity-color: var(--color-error); }
  .entity-item-people { --entity-color: var(--color-success); }
  .entity-item-organizations { --entity-color: var(--color-accent); }
  .entity-item-locations { --entity-color: var(--color-info); }
  .entity-item-documents { --entity-color: var(--color-warning); }
</style>