<script>
  import { onMount } from 'svelte';
  import { featuresStore, tasksStore, notesStore } from '../stores.js';
  
  let selectedEntityType = 'all';
  let entityCounts = {};
  
  const entityTypes = [
    { id: 'all', label: 'All Entities', icon: '🔗', color: '#9b59d0' },
    { id: 'features', label: 'Features', icon: '⚡', color: '#f59e0b' },
    { id: 'tasks', label: 'Tasks', icon: '✓', color: '#4ade80' },
    { id: 'sessions', label: 'Sessions', icon: '🎬', color: '#06b6d4' },
    { id: 'notes', label: 'Notes', icon: '📝', color: '#8b5cf6' },
    { id: 'milestones', label: 'Milestones', icon: '🎯', color: '#ef4444' }
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

<div class="entity-explorer-card">
  <h2>Entity Explorer</h2>
  
  <div class="entity-filters">
    {#each entityTypes as entity}
      <button 
        class="entity-filter" 
        class:active={selectedEntityType === entity.id}
        style="--entity-color: {entity.color}"
        on:click={() => selectEntityType(entity.id)}
      >
        <span class="entity-icon">{entity.icon}</span>
        <span class="entity-label">{entity.label}</span>
        {#if entity.id !== 'all'}
          <span class="entity-count">{entityCounts[entity.id] || 0}</span>
        {/if}
      </button>
    {/each}
  </div>

  <div class="entity-stats">
    <div class="total-entities">
      <div class="total-count">{entityCounts.total}</div>
      <div class="total-label">Total Entities</div>
    </div>
    
    <div class="entity-breakdown">
      {#each entityTypes.slice(1) as entity}
        <div class="breakdown-item" style="--entity-color: {entity.color}">
          <div class="breakdown-bar">
            <div class="breakdown-fill" style="width: {((entityCounts[entity.id] || 0) / entityCounts.total * 100)}%"></div>
          </div>
          <div class="breakdown-info">
            <span class="breakdown-label">{entity.label}</span>
            <span class="breakdown-count">{entityCounts[entity.id] || 0}</span>
          </div>
        </div>
      {/each}
    </div>
  </div>

  <div class="relationship-summary">
    <h3>Entity Relationships</h3>
    <div class="relationship-stats">
      <div class="relationship-item">
        <span class="relationship-label">Dependencies</span>
        <span class="relationship-count">156</span>
      </div>
      <div class="relationship-item">
        <span class="relationship-label">References</span>
        <span class="relationship-count">89</span>
      </div>
      <div class="relationship-item">
        <span class="relationship-label">Blocks</span>
        <span class="relationship-count">12</span>
      </div>
    </div>
    
    <div class="network-preview">
      <div class="network-node features" style="top: 20%; left: 20%"></div>
      <div class="network-node tasks" style="top: 40%; left: 60%"></div>
      <div class="network-node sessions" style="top: 70%; left: 30%"></div>
      <div class="network-node notes" style="top: 60%; left: 80%"></div>
      <div class="network-connection" style="top: 25%; left: 25%; width: 35%; transform: rotate(15deg)"></div>
      <div class="network-connection" style="top: 45%; left: 40%; width: 25%; transform: rotate(-20deg)"></div>
      <div class="network-connection" style="top: 65%; left: 50%; width: 30%; transform: rotate(35deg)"></div>
    </div>
  </div>
</div>

<style>
  .entity-explorer-card {
    background: linear-gradient(135deg, #1a1a1a 0%, #2a2a2a 100%);
    border: 1px solid #333;
    border-radius: 12px;
    padding: 24px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  .entity-explorer-card h2 {
    color: #9b59d0;
    margin: 0 0 20px 0;
    font-size: 20px;
    font-weight: 600;
  }

  .entity-filters {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-bottom: 20px;
  }

  .entity-filter {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 12px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(var(--entity-color-rgb, 155, 89, 208), 0.3);
    color: #ccc;
    border-radius: 20px;
    font-size: 12px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .entity-filter:hover {
    background: rgba(var(--entity-color-rgb, 155, 89, 208), 0.1);
    border-color: var(--entity-color);
  }

  .entity-filter.active {
    background: rgba(var(--entity-color-rgb, 155, 89, 208), 0.2);
    border-color: var(--entity-color);
    color: var(--entity-color);
  }

  .entity-icon {
    font-size: 14px;
  }

  .entity-count {
    background: rgba(255, 255, 255, 0.1);
    padding: 2px 6px;
    border-radius: 10px;
    font-size: 10px;
    font-weight: 600;
  }

  .entity-stats {
    margin-bottom: 24px;
  }

  .total-entities {
    text-align: center;
    margin-bottom: 16px;
    padding-bottom: 16px;
    border-bottom: 1px solid #333;
  }

  .total-count {
    font-size: 32px;
    font-weight: 700;
    color: #9b59d0;
    line-height: 1;
  }

  .total-label {
    font-size: 12px;
    color: #888;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .entity-breakdown {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .breakdown-item {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .breakdown-bar {
    flex: 1;
    height: 4px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 2px;
    overflow: hidden;
  }

  .breakdown-fill {
    height: 100%;
    background: var(--entity-color);
    transition: width 0.6s ease;
  }

  .breakdown-info {
    display: flex;
    gap: 8px;
    align-items: center;
    min-width: 80px;
  }

  .breakdown-label {
    font-size: 11px;
    color: #888;
  }

  .breakdown-count {
    font-size: 11px;
    color: var(--entity-color);
    font-weight: 600;
    font-family: monospace;
  }

  .relationship-summary {
    border-top: 1px solid #333;
    padding-top: 20px;
  }

  .relationship-summary h3 {
    color: #ccc;
    font-size: 14px;
    font-weight: 600;
    margin: 0 0 16px 0;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .relationship-stats {
    display: flex;
    justify-content: space-between;
    margin-bottom: 16px;
  }

  .relationship-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    font-size: 12px;
  }

  .relationship-label {
    color: #888;
  }

  .relationship-count {
    color: #9b59d0;
    font-weight: 600;
    font-size: 14px;
  }

  .network-preview {
    position: relative;
    height: 100px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    overflow: hidden;
  }

  .network-node {
    position: absolute;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    animation: pulse 2s infinite;
  }

  .network-node.features {
    background: #f59e0b;
  }

  .network-node.tasks {
    background: #4ade80;
  }

  .network-node.sessions {
    background: #06b6d4;
  }

  .network-node.notes {
    background: #8b5cf6;
  }

  .network-connection {
    position: absolute;
    height: 1px;
    background: linear-gradient(90deg, rgba(155, 89, 208, 0.6), transparent);
    transform-origin: left center;
  }

  @keyframes pulse {
    0%, 100% { opacity: 0.6; transform: scale(1); }
    50% { opacity: 1; transform: scale(1.2); }
  }
</style>