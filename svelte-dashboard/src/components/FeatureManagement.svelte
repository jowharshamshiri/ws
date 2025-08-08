<script>
  import { onMount } from 'svelte';
  import { featuresStore } from '../stores.js';
  import { apiService } from '../services/api.js';
  
  let features = [];
  let selectedFeature = null;
  let filterState = 'all';
  let searchTerm = '';
  
  // Kanban columns configuration
  const kanbanColumns = [
    { id: 'pending', title: '❌ Not Implemented', color: '#666' },
    { id: 'implemented', title: '🟠 Implemented', color: '#f59e0b' },
    { id: 'testing', title: '🟡 Testing', color: '#eab308' },
    { id: 'completed', title: '🟢 Completed', color: '#4ade80' },
    { id: 'warning', title: '⚠️ Needs Repair', color: '#f97316' }
  ];

  $: features = $featuresStore || generateSampleFeatures();
  
  $: filteredFeatures = features.filter(feature => {
    const matchesState = filterState === 'all' || getFeatureColumn(feature.state) === filterState;
    const matchesSearch = !searchTerm || 
      feature.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
      feature.description.toLowerCase().includes(searchTerm.toLowerCase());
    return matchesState && matchesSearch;
  });

  $: featuresByColumn = kanbanColumns.reduce((acc, column) => {
    acc[column.id] = filteredFeatures.filter(feature => 
      getFeatureColumn(feature.state) === column.id
    );
    return acc;
  }, {});

  function generateSampleFeatures() {
    return [
      { id: 'F0200', name: 'ADE Overview Dashboard', description: 'Main dashboard with project metrics and AI agent status', state: '🟢', category: 'ADE Interface' },
      { id: 'F0201', name: 'Active Session Indicator', description: 'Live status showing current AI agent and working context', state: '🟠', category: 'ADE Interface' },
      { id: 'F0202', name: 'Implementation Progress Card', description: 'Visual progress indicator for feature completion', state: '🟠', category: 'ADE Interface' },
      { id: 'F0228', name: 'Feature Kanban Board', description: 'Drag-and-drop kanban board with feature state columns', state: '🟡', category: 'ADE Interface' },
      { id: 'F0229', name: 'Feature State Legend', description: 'Visual legend explaining feature state indicators', state: '❌', category: 'ADE Interface' },
      { id: 'F0001', name: 'Unified Command Line Interface', description: 'Single ws binary consolidating all tool functionalities', state: '🟢', category: 'Core Tools' }
    ];
  }

  function getFeatureColumn(state) {
    switch (state) {
      case '❌': return 'pending';
      case '🟠': return 'implemented';
      case '🟡': return 'testing';
      case '🟢': return 'completed';
      case '⚠️': return 'warning';
      default: return 'pending';
    }
  }

  function selectFeature(feature) {
    selectedFeature = selectedFeature?.id === feature.id ? null : feature;
  }

  function updateFeatureState(featureId, newState) {
    // Update feature state (would call API in real implementation)
    featuresStore.update(features => 
      features.map(f => f.id === featureId ? { ...f, state: newState } : f)
    );
  }

  onMount(() => {
    // Load features from API if store is empty
    if ($featuresStore.length === 0) {
      featuresStore.set(generateSampleFeatures());
    }
  });
</script>

<div class="feature-management">
  <div class="feature-header">
    <h1>Feature Management</h1>
    
    <div class="feature-controls">
      <div class="search-box">
        <input 
          type="text" 
          placeholder="Search features..." 
          bind:value={searchTerm}
        />
      </div>
      
      <div class="filter-buttons">
        <button 
          class="filter-btn" 
          class:active={filterState === 'all'}
          on:click={() => filterState = 'all'}
        >
          All ({features.length})
        </button>
        {#each kanbanColumns as column}
          <button 
            class="filter-btn" 
            class:active={filterState === column.id}
            style="--column-color: {column.color}"
            on:click={() => filterState = column.id}
          >
            {column.title.split(' ')[0]} ({featuresByColumn[column.id]?.length || 0})
          </button>
        {/each}
      </div>
    </div>
  </div>

  <div class="kanban-container">
    <div class="kanban-board">
      {#each kanbanColumns as column}
        <div class="kanban-column" style="--column-color: {column.color}">
          <div class="column-header">
            <h3>{column.title}</h3>
            <span class="column-count">{featuresByColumn[column.id]?.length || 0}</span>
          </div>
          
          <div class="column-content">
            {#each (featuresByColumn[column.id] || []) as feature}
              <div 
                class="feature-card" 
                class:selected={selectedFeature?.id === feature.id}
                on:click={() => selectFeature(feature)}
              >
                <div class="feature-id">{feature.id}</div>
                <div class="feature-name">{feature.name}</div>
                <div class="feature-description">{feature.description}</div>
                <div class="feature-meta">
                  <span class="feature-category">{feature.category}</span>
                  <span class="feature-state">{feature.state}</span>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/each}
    </div>
  </div>

  {#if selectedFeature}
    <div class="feature-detail-panel">
      <div class="detail-header">
        <h2>{selectedFeature.name}</h2>
        <button class="close-btn" on:click={() => selectedFeature = null}>×</button>
      </div>
      
      <div class="detail-content">
        <div class="detail-item">
          <label>ID:</label>
          <span class="feature-id">{selectedFeature.id}</span>
        </div>
        
        <div class="detail-item">
          <label>Description:</label>
          <p>{selectedFeature.description}</p>
        </div>
        
        <div class="detail-item">
          <label>Category:</label>
          <span>{selectedFeature.category}</span>
        </div>
        
        <div class="detail-item">
          <label>Status:</label>
          <div class="status-controls">
            {#each ['❌', '🟠', '🟡', '🟢', '⚠️'] as state}
              <button 
                class="status-btn" 
                class:active={selectedFeature.state === state}
                on:click={() => updateFeatureState(selectedFeature.id, state)}
              >
                {state}
              </button>
            {/each}
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .feature-management {
    padding: 20px;
    min-height: 100vh;
    background: #0a0a0b;
    position: relative;
  }

  .feature-header {
    margin-bottom: 24px;
  }

  .feature-header h1 {
    color: #9b59d0;
    font-size: 28px;
    font-weight: 600;
    margin-bottom: 16px;
  }

  .feature-controls {
    display: flex;
    gap: 20px;
    align-items: center;
    flex-wrap: wrap;
  }

  .search-box input {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid #333;
    border-radius: 8px;
    padding: 10px 16px;
    color: #fff;
    font-size: 14px;
    min-width: 250px;
  }

  .search-box input::placeholder {
    color: #888;
  }

  .filter-buttons {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .filter-btn {
    padding: 8px 12px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid #333;
    color: #888;
    border-radius: 6px;
    font-size: 12px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .filter-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #ccc;
  }

  .filter-btn.active {
    background: rgba(155, 89, 208, 0.2);
    border-color: #9b59d0;
    color: #9b59d0;
  }

  .kanban-container {
    overflow-x: auto;
    padding-bottom: 20px;
  }

  .kanban-board {
    display: flex;
    gap: 16px;
    min-width: max-content;
  }

  .kanban-column {
    background: linear-gradient(135deg, #1a1a1a 0%, #2a2a2a 100%);
    border: 1px solid #333;
    border-radius: 12px;
    min-width: 300px;
    max-width: 320px;
  }

  .column-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid #333;
  }

  .column-header h3 {
    color: var(--column-color);
    font-size: 14px;
    font-weight: 600;
    margin: 0;
  }

  .column-count {
    background: rgba(255, 255, 255, 0.1);
    color: var(--column-color);
    padding: 4px 8px;
    border-radius: 12px;
    font-size: 11px;
    font-weight: 600;
  }

  .column-content {
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-height: 400px;
  }

  .feature-card {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    padding: 16px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .feature-card:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(155, 89, 208, 0.3);
  }

  .feature-card.selected {
    border-color: #9b59d0;
    background: rgba(155, 89, 208, 0.1);
  }

  .feature-id {
    font-family: monospace;
    font-size: 12px;
    color: #9b59d0;
    font-weight: 600;
    margin-bottom: 4px;
  }

  .feature-name {
    font-size: 14px;
    font-weight: 600;
    color: #fff;
    margin-bottom: 8px;
    line-height: 1.3;
  }

  .feature-description {
    font-size: 12px;
    color: #888;
    line-height: 1.4;
    margin-bottom: 12px;
  }

  .feature-meta {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .feature-category {
    font-size: 10px;
    color: #666;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .feature-state {
    font-size: 14px;
  }

  .feature-detail-panel {
    position: fixed;
    right: 20px;
    top: 50%;
    transform: translateY(-50%);
    width: 350px;
    background: linear-gradient(135deg, #1a1a1a 0%, #2a2a2a 100%);
    border: 1px solid #333;
    border-radius: 12px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    z-index: 10;
  }

  .detail-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px;
    border-bottom: 1px solid #333;
  }

  .detail-header h2 {
    color: #9b59d0;
    font-size: 16px;
    font-weight: 600;
    margin: 0;
    max-width: 280px;
    line-height: 1.3;
  }

  .close-btn {
    background: none;
    border: none;
    color: #888;
    font-size: 20px;
    cursor: pointer;
    padding: 0;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .close-btn:hover {
    color: #ccc;
  }

  .detail-content {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .detail-item label {
    display: block;
    color: #888;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 8px;
  }

  .detail-item span,
  .detail-item p {
    color: #fff;
    font-size: 14px;
    line-height: 1.4;
    margin: 0;
  }

  .status-controls {
    display: flex;
    gap: 8px;
  }

  .status-btn {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid #333;
    color: #fff;
    padding: 8px 12px;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.2s ease;
    font-size: 14px;
  }

  .status-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: #555;
  }

  .status-btn.active {
    background: rgba(155, 89, 208, 0.2);
    border-color: #9b59d0;
  }
</style>