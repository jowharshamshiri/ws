<script>
  import { onMount } from 'svelte';
  import { featuresStore, tasksStore, notesStore } from '../stores.js';
  import TaskKanban from './TaskKanban.svelte';
  
  let selectedEntityType = 'all';
  let entityCounts = {};
  
  const entityTypes = [
    { id: 'all', label: 'All Entities', icon: 'A' },
    { id: 'features', label: 'Features', icon: 'F' },
    { id: 'tasks', label: 'Tasks', icon: 'T' },
    { id: 'sessions', label: 'Sessions', icon: 'S' },
    { id: 'notes', label: 'Notes', icon: 'N' },
    { id: 'milestones', label: 'Milestones', icon: 'M' }
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

{#if selectedEntityType === 'tasks'}
  <TaskKanban />
{:else}
  <div class="entity-explorer">
    <h2>Entity Explorer</h2>
    
    <div class="entity-filters">
      {#each entityTypes as entity}
        <button 
          class="entity-filter" 
          class:active={selectedEntityType === entity.id}
          on:click={() => selectEntityType(entity.id)}
        >
          <span>{entity.icon}</span>
          <span>{entity.label}</span>
          {#if entity.id !== 'all'}
            <span>{entityCounts[entity.id] || 0}</span>
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
          <div class="breakdown-item">
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
    </div>
  </div>
{/if}

