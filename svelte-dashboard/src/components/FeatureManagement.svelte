<script>
  import { onMount } from 'svelte';
  import { featuresStore } from '../stores.js';
  
  let features = [];
  let selectedFeature = null;
  let filterCategory = 'all';
  let showCreateDialog = false;
  let viewMode = 'kanban'; // kanban, timeline, dependencies
  
  // Sample feature data with different states
  const sampleFeatures = [
    {
      id: 'F0228',
      title: 'Feature Kanban Board',
      description: 'Drag-and-drop kanban board with feature state columns',
      state: '🟢',
      category: 'ADE Interface',
      priority: 'high',
      dependencies: [],
      assignee: 'Claude',
      estimatedHours: 8,
      completedHours: 8,
      tags: ['kanban', 'ui', 'drag-drop']
    },
    {
      id: 'F0229',
      title: 'Feature State Legend',
      description: 'Visual legend explaining feature state indicators',
      state: '🟠',
      category: 'ADE Interface', 
      priority: 'medium',
      dependencies: ['F0228'],
      assignee: 'Claude',
      estimatedHours: 4,
      completedHours: 3,
      tags: ['legend', 'ui', 'documentation']
    },
    {
      id: 'F0230',
      title: 'Feature Card Details',
      description: 'Individual feature cards with ID, description, status',
      state: '🟡',
      category: 'ADE Interface',
      priority: 'high',
      dependencies: ['F0228', 'F0229'],
      assignee: 'Claude',
      estimatedHours: 6,
      completedHours: 6,
      tags: ['cards', 'details', 'ui']
    },
    {
      id: 'F0231',
      title: 'Feature Dependencies Graph',
      description: 'Visual graph showing feature relationships and dependencies',
      state: '❌',
      category: 'ADE Interface',
      priority: 'medium',
      dependencies: ['F0230'],
      assignee: 'Claude',
      estimatedHours: 12,
      completedHours: 0,
      tags: ['graph', 'dependencies', 'visualization']
    },
    {
      id: 'F0232',
      title: 'Implementation Velocity Chart',
      description: 'Weekly progress chart with completion predictions',
      state: '🟠',
      category: 'Analytics',
      priority: 'low',
      dependencies: [],
      assignee: 'Claude',
      estimatedHours: 10,
      completedHours: 4,
      tags: ['analytics', 'chart', 'velocity']
    },
    {
      id: 'F0233',
      title: 'Feature Category Filtering',
      description: 'Filter features by category (Core, Entity, Dashboard, etc.)',
      state: '⚠️',
      category: 'Core',
      priority: 'high',
      dependencies: [],
      assignee: 'Claude',
      estimatedHours: 3,
      completedHours: 3,
      tags: ['filtering', 'categories', 'ui']
    }
  ];

  $: features = $featuresStore.length > 0 ? $featuresStore : sampleFeatures;
  $: filteredFeatures = features.filter(feature => 
    filterCategory === 'all' || feature.category === filterCategory
  );
  $: featuresByState = {
    '❌': filteredFeatures.filter(f => f.state === '❌'),
    '🟠': filteredFeatures.filter(f => f.state === '🟠'), 
    '🟡': filteredFeatures.filter(f => f.state === '🟡'),
    '🟢': filteredFeatures.filter(f => f.state === '🟢'),
    '⚠️': filteredFeatures.filter(f => f.state === '⚠️'),
    '🔴': filteredFeatures.filter(f => f.state === '🔴')
  };
  $: categories = [...new Set(features.map(f => f.category))];
  $: velocityData = calculateVelocity();

  function getStateLabel(state) {
    switch (state) {
      case '❌': return 'Not Started';
      case '🟠': return 'In Progress';
      case '🟡': return 'Testing';
      case '🟢': return 'Complete';
      case '⚠️': return 'Issues';
      case '🔴': return 'Blocked';
      default: return 'Unknown';
    }
  }

  function getStateColor(state) {
    switch (state) {
      case '❌': return '#6b7280';
      case '🟠': return '#f59e0b';
      case '🟡': return '#eab308';
      case '🟢': return '#10b981';
      case '⚠️': return '#f97316';
      case '🔴': return '#ef4444';
      default: return '#6b7280';
    }
  }

  function getPriorityColor(priority) {
    switch (priority) {
      case 'high': return '#ef4444';
      case 'medium': return '#f59e0b';
      case 'low': return '#10b981';
      default: return '#6b7280';
    }
  }

  function selectFeature(feature) {
    selectedFeature = feature;
  }

  function calculateVelocity() {
    // Calculate weekly velocity based on completed features
    const completedFeatures = features.filter(f => f.state === '🟢');
    const weeksData = [
      { week: 'W1', completed: 12 },
      { week: 'W2', completed: 18 },
      { week: 'W3', completed: 15 },
      { week: 'W4', completed: 21 },
      { week: 'W5', completed: 19 }
    ];
    return weeksData;
  }

  function getCompletionPercentage(feature) {
    if (feature.estimatedHours === 0) return 0;
    return Math.min(100, Math.round((feature.completedHours / feature.estimatedHours) * 100));
  }

  function handleDragStart(event, feature) {
    event.dataTransfer.setData('text/plain', JSON.stringify(feature));
    event.dataTransfer.effectAllowed = 'move';
  }

  function handleDragOver(event) {
    event.preventDefault();
    event.dataTransfer.dropEffect = 'move';
  }

  function handleDrop(event, newState) {
    event.preventDefault();
    const featureData = event.dataTransfer.getData('text/plain');
    const feature = JSON.parse(featureData);
    
    // Update feature state
    const featureIndex = features.findIndex(f => f.id === feature.id);
    if (featureIndex !== -1) {
      features[featureIndex].state = newState;
      features = [...features]; // Trigger reactivity
    }
  }

  function createFeature() {
    showCreateDialog = true;
  }

  function closeCreateDialog() {
    showCreateDialog = false;
  }

  onMount(() => {
    if (features.length > 0 && !selectedFeature) {
      selectFeature(features[0]);
    }
  });
</script>

<div class="feature-management">
  <div class="feature-header">
    <h1>Feature Management</h1>
    
    <div class="header-controls">
      <div class="view-toggle">
        <button 
          class="toggle-btn" 
          class:active={viewMode === 'kanban'}
          on:click={() => viewMode = 'kanban'}
        >
          Kanban
        </button>
        <button 
          class="toggle-btn" 
          class:active={viewMode === 'timeline'}
          on:click={() => viewMode = 'timeline'}
        >
          Timeline
        </button>
        <button 
          class="toggle-btn" 
          class:active={viewMode === 'dependencies'}
          on:click={() => viewMode = 'dependencies'}
        >
          Dependencies
        </button>
      </div>
      
      <div class="header-filters">
        <select bind:value={filterCategory} class="category-filter">
          <option value="all">All Categories</option>
          {#each categories as category}
            <option value={category}>{category}</option>
          {/each}
        </select>
        
        <button class="create-btn" on:click={createFeature}>
          + Add Feature
        </button>
      </div>
    </div>
  </div>

  <div class="feature-stats">
    <div class="stat-card">
      <div class="stat-number">{filteredFeatures.length}</div>
      <div class="stat-label">Total Features</div>
    </div>
    <div class="stat-card complete">
      <div class="stat-number">{featuresByState['🟢'].length}</div>
      <div class="stat-label">Complete</div>
    </div>
    <div class="stat-card progress">
      <div class="stat-number">{featuresByState['🟠'].length + featuresByState['🟡'].length}</div>
      <div class="stat-label">In Progress</div>
    </div>
    <div class="stat-card blocked">
      <div class="stat-number">{featuresByState['🔴'].length + featuresByState['⚠️'].length}</div>
      <div class="stat-label">Issues</div>
    </div>
    <div class="stat-card velocity">
      <div class="stat-number">17.2</div>
      <div class="stat-label">Features/Week</div>
    </div>
  </div>

  <!-- F0229: Feature State Legend -->
  <div class="feature-legend">
    <h3>Feature State Legend</h3>
    <div class="legend-items">
      <div class="legend-item">
        <span class="legend-icon" style="color: {getStateColor('❌')}">❌</span>
        <div class="legend-info">
          <div class="legend-title">Not Started</div>
          <div class="legend-desc">Feature not yet implemented</div>
        </div>
      </div>
      <div class="legend-item">
        <span class="legend-icon" style="color: {getStateColor('🟠')}">🟠</span>
        <div class="legend-info">
          <div class="legend-title">In Progress</div>
          <div class="legend-desc">Implementation in code but lacks dedicated tests</div>
        </div>
      </div>
      <div class="legend-item">
        <span class="legend-icon" style="color: {getStateColor('🟡')}">🟡</span>
        <div class="legend-info">
          <div class="legend-title">Testing</div>
          <div class="legend-desc">Implemented with dedicated FAILING tests</div>
        </div>
      </div>
      <div class="legend-item">
        <span class="legend-icon" style="color: {getStateColor('🟢')}">🟢</span>
        <div class="legend-info">
          <div class="legend-title">Complete</div>
          <div class="legend-desc">Implemented with dedicated PASSING tests</div>
        </div>
      </div>
      <div class="legend-item">
        <span class="legend-icon" style="color: {getStateColor('⚠️')}">⚠️</span>
        <div class="legend-info">
          <div class="legend-title">Issues</div>
          <div class="legend-desc">Has tests but they are fake/tautological/broken</div>
        </div>
      </div>
      <div class="legend-item">
        <span class="legend-icon" style="color: {getStateColor('🔴')}">🔴</span>
        <div class="legend-info">
          <div class="legend-title">Critical Issue</div>
          <div class="legend-desc">Critical issue requiring immediate attention</div>
        </div>
      </div>
    </div>
  </div>

  {#if viewMode === 'kanban'}
    <div class="kanban-board">
      {#each Object.entries(featuresByState) as [state, stateFeatures]}
        <div 
          class="kanban-column"
          on:dragover={handleDragOver}
          on:drop={(e) => handleDrop(e, state)}
        >
          <div class="column-header" style="border-color: {getStateColor(state)}">
            <div class="column-title">
              <span class="state-icon">{state}</span>
              {getStateLabel(state)}
            </div>
            <div class="column-count">{stateFeatures.length}</div>
          </div>
          
          <div class="column-content">
            {#each stateFeatures as feature}
              <div 
                class="feature-card"
                class:selected={selectedFeature?.id === feature.id}
                draggable="true"
                on:dragstart={(e) => handleDragStart(e, feature)}
                on:click={() => selectFeature(feature)}
              >
                <div class="card-header">
                  <div class="feature-id">{feature.id}</div>
                  <div 
                    class="priority-badge"
                    style="background-color: {getPriorityColor(feature.priority)}"
                  >
                    {feature.priority.toUpperCase()}
                  </div>
                </div>
                
                <div class="feature-title">{feature.title}</div>
                <div class="feature-description">{feature.description}</div>
                
                <div class="feature-meta">
                  <div class="feature-progress">
                    <div class="progress-bar">
                      <div 
                        class="progress-fill" 
                        style="width: {getCompletionPercentage(feature)}%; background-color: {getStateColor(feature.state)}"
                      ></div>
                    </div>
                    <span class="progress-text">{getCompletionPercentage(feature)}%</span>
                  </div>
                  
                  <!-- F0230: Enhanced Card Details -->
                  <div class="feature-details">
                    <div class="detail-row">
                      <div class="detail-item">
                        <span class="detail-label">Effort:</span>
                        <span class="detail-value">{feature.completedHours}h / {feature.estimatedHours}h</span>
                      </div>
                      {#if feature.assignee}
                        <div class="detail-item">
                          <span class="detail-label">Assignee:</span>
                          <span class="detail-value">{feature.assignee}</span>
                        </div>
                      {/if}
                    </div>
                    {#if feature.category}
                      <div class="detail-row">
                        <div class="category-badge">{feature.category}</div>
                      </div>
                    {/if}
                  </div>
                  
                  <div class="feature-tags">
                    {#each feature.tags.slice(0, 2) as tag}
                      <span class="tag">{tag}</span>
                    {/each}
                    {#if feature.tags.length > 2}
                      <span class="tag-more">+{feature.tags.length - 2}</span>
                    {/if}
                  </div>
                </div>
                
                {#if feature.dependencies.length > 0}
                  <div class="dependencies">
                    <span class="dep-label">Depends on:</span>
                    {#each feature.dependencies.slice(0, 2) as dep}
                      <span class="dependency">{dep}</span>
                    {/each}
                    {#if feature.dependencies.length > 2}
                      <span class="dep-more">+{feature.dependencies.length - 2}</span>
                    {/if}
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        </div>
      {/each}
    </div>
  {:else if viewMode === 'dependencies'}
    <div class="dependencies-view">
      <div class="graph-container">
        <div class="graph-header">
          <h3>Feature Dependencies Graph</h3>
          <div class="graph-controls">
            <button class="graph-btn">Reset View</button>
            <button class="graph-btn">Auto Layout</button>
          </div>
        </div>
        
        <!-- F0231: Enhanced Feature Dependencies Graph -->
        <div class="dependency-graph">
          <svg width="100%" height="500" viewBox="0 0 1000 500">
            <!-- Feature nodes with enhanced positioning -->
            {#each filteredFeatures as feature, i}
              <g 
                class="feature-node"
                transform="translate({120 + (i % 5) * 160}, {100 + Math.floor(i / 5) * 120})"
                on:click={() => selectFeature(feature)}
              >
                <!-- Node background -->
                <circle
                  cx="0"
                  cy="0" 
                  r="30"
                  fill={getStateColor(feature.state)}
                  stroke={selectedFeature?.id === feature.id ? '#9b59d0' : '#333'}
                  stroke-width={selectedFeature?.id === feature.id ? '3' : '2'}
                  class="node-circle"
                />
                
                <!-- Feature ID -->
                <text
                  x="0"
                  y="-5"
                  text-anchor="middle"
                  fill="#fff"
                  font-size="8"
                  font-weight="700"
                  font-family="monospace"
                >
                  {feature.id}
                </text>
                
                <!-- Priority indicator -->
                <circle
                  cx="18"
                  cy="-18"
                  r="6"
                  fill={getPriorityColor(feature.priority)}
                  stroke="#000"
                  stroke-width="1"
                />
                
                <!-- Feature title -->
                <text
                  x="0"
                  y="50"
                  text-anchor="middle"
                  fill="#aaa"
                  font-size="10"
                  font-weight="500"
                >
                  {feature.title.length > 20 ? feature.title.slice(0, 17) + '...' : feature.title}
                </text>
                
                <!-- Progress indicator -->
                <rect
                  x="-25"
                  y="20"
                  width="50"
                  height="4"
                  fill="rgba(255, 255, 255, 0.1)"
                  rx="2"
                />
                <rect
                  x="-25"
                  y="20"
                  width="{getCompletionPercentage(feature) / 2}"
                  height="4"
                  fill={getStateColor(feature.state)}
                  rx="2"
                />
              </g>
            {/each}
            
            <!-- Dynamic dependency lines -->
            {#each filteredFeatures as feature, i}
              {#each feature.dependencies as depId}
                {#each filteredFeatures as depFeature, j}
                  {#if depFeature.id === depId}
                    <line
                      x1={120 + (j % 5) * 160}
                      y1={100 + Math.floor(j / 5) * 120}
                      x2={120 + (i % 5) * 160}
                      y2={100 + Math.floor(i / 5) * 120}
                      stroke="#666"
                      stroke-width="2"
                      stroke-dasharray="5,5"
                      marker-end="url(#arrowhead)"
                      class="dependency-line"
                    />
                    
                    <!-- Dependency strength indicator -->
                    <circle
                      cx={(120 + (j % 5) * 160 + 120 + (i % 5) * 160) / 2}
                      cy={(100 + Math.floor(j / 5) * 120 + 100 + Math.floor(i / 5) * 120) / 2}
                      r="3"
                      fill="#f59e0b"
                      stroke="#333"
                      stroke-width="1"
                    />
                  {/if}
                {/each}
              {/each}
            {/each}
            
            <defs>
              <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
                <polygon points="0 0, 10 3.5, 0 7" fill="#666" />
              </marker>
            </defs>
          </svg>
          
          <!-- Graph legend -->
          <div class="graph-legend">
            <div class="legend-row">
              <div class="legend-item-small">
                <div class="legend-circle" style="background: #ef4444;"></div>
                <span>High Priority</span>
              </div>
              <div class="legend-item-small">
                <div class="legend-circle" style="background: #f59e0b;"></div>
                <span>Medium Priority</span>
              </div>
              <div class="legend-item-small">
                <div class="legend-circle" style="background: #10b981;"></div>
                <span>Low Priority</span>
              </div>
            </div>
            <div class="legend-row">
              <div class="legend-item-small">
                <div class="legend-line"></div>
                <span>Dependencies</span>
              </div>
              <div class="legend-item-small">
                <div class="legend-dot"></div>
                <span>Connection Point</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  {:else if viewMode === 'timeline'}
    <div class="timeline-view">
      <div class="velocity-chart">
        <h3>Implementation Velocity</h3>
        <div class="chart-container">
          {#each velocityData as week}
            <div class="velocity-bar">
              <div 
                class="bar-fill" 
                style="height: {(week.completed / 25) * 100}%"
              ></div>
              <div class="bar-label">{week.week}</div>
              <div class="bar-value">{week.completed}</div>
            </div>
          {/each}
        </div>
        
        <div class="completion-prediction">
          <div class="prediction-card">
            <div class="prediction-title">Estimated Completion</div>
            <div class="prediction-date">2 weeks</div>
            <div class="prediction-confidence">85% confidence</div>
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>

<!-- Feature Creation Dialog -->
{#if showCreateDialog}
  <div class="dialog-overlay" on:click={closeCreateDialog}>
    <div class="dialog" on:click|stopPropagation>
      <div class="dialog-header">
        <h3>Add New Feature</h3>
        <button class="close-btn" on:click={closeCreateDialog}>×</button>
      </div>
      
      <div class="dialog-content">
        <div class="form-group">
          <label>Feature Title</label>
          <input type="text" placeholder="Enter feature title..." />
        </div>
        
        <div class="form-group">
          <label>Description</label>
          <textarea placeholder="Describe the feature..."></textarea>
        </div>
        
        <div class="form-row">
          <div class="form-group">
            <label>Category</label>
            <select>
              <option value="ADE Interface">ADE Interface</option>
              <option value="Core">Core</option>
              <option value="Analytics">Analytics</option>
            </select>
          </div>
          
          <div class="form-group">
            <label>Priority</label>
            <select>
              <option value="high">High</option>
              <option value="medium">Medium</option>
              <option value="low">Low</option>
            </select>
          </div>
        </div>
        
        <div class="form-group">
          <label>Estimated Hours</label>
          <input type="number" min="1" max="100" value="8" />
        </div>
      </div>
      
      <div class="dialog-actions">
        <button class="cancel-btn" on:click={closeCreateDialog}>Cancel</button>
        <button class="create-btn">Create Feature</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .feature-management {
    padding: 20px;
    min-height: 100vh;
    background: #0a0a0b;
    color: #fff;
  }

  .feature-header {
    margin-bottom: 24px;
  }

  .feature-header h1 {
    color: #9b59d0;
    font-size: 28px;
    font-weight: 600;
    margin: 0 0 16px 0;
  }

  .header-controls {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 20px;
  }

  .view-toggle {
    display: flex;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    padding: 2px;
  }

  .toggle-btn {
    padding: 6px 12px;
    background: none;
    border: none;
    color: #888;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .toggle-btn.active {
    background: #9b59d0;
    color: #fff;
  }

  .header-filters {
    display: flex;
    gap: 12px;
    align-items: center;
  }

  .category-filter {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid #333;
    color: #fff;
    padding: 6px 10px;
    border-radius: 4px;
    font-size: 12px;
  }

  .create-btn {
    background: rgba(155, 89, 208, 0.2);
    border: 1px solid #9b59d0;
    color: #9b59d0;
    padding: 6px 12px;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .create-btn:hover {
    background: rgba(155, 89, 208, 0.3);
  }

  .feature-stats {
    display: flex;
    gap: 16px;
    margin-bottom: 24px;
    flex-wrap: wrap;
  }

  .stat-card {
    background: linear-gradient(135deg, #1a1a1a 0%, #2a2a2a 100%);
    border: 1px solid #333;
    border-radius: 8px;
    padding: 16px;
    min-width: 100px;
    text-align: center;
  }

  .stat-card.complete {
    border-color: #10b981;
  }

  .stat-card.progress {
    border-color: #f59e0b;
  }

  .stat-card.blocked {
    border-color: #ef4444;
  }

  .stat-card.velocity {
    border-color: #9b59d0;
  }

  .stat-number {
    font-size: 20px;
    font-weight: 700;
    margin-bottom: 4px;
    color: #fff;
  }

  .stat-card.complete .stat-number { color: #10b981; }
  .stat-card.progress .stat-number { color: #f59e0b; }
  .stat-card.blocked .stat-number { color: #ef4444; }
  .stat-card.velocity .stat-number { color: #9b59d0; }

  .stat-label {
    color: #888;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .kanban-board {
    display: grid;
    grid-template-columns: repeat(6, 1fr);
    gap: 16px;
    min-height: 600px;
  }

  .kanban-column {
    background: linear-gradient(135deg, #1a1a1a 0%, #2a2a2a 100%);
    border: 1px solid #333;
    border-radius: 12px;
    overflow: hidden;
  }

  .column-header {
    padding: 12px 16px;
    border-bottom: 2px solid;
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: rgba(255, 255, 255, 0.02);
  }

  .column-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    font-weight: 600;
    color: #fff;
  }

  .state-icon {
    font-size: 14px;
  }

  .column-count {
    background: rgba(255, 255, 255, 0.1);
    color: #888;
    padding: 2px 8px;
    border-radius: 12px;
    font-size: 11px;
    font-weight: 600;
  }

  .column-content {
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    max-height: 500px;
    overflow-y: auto;
  }

  .feature-card {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid #333;
    border-radius: 8px;
    padding: 12px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .feature-card:hover {
    background: rgba(255, 255, 255, 0.08);
    border-color: #555;
  }

  .feature-card.selected {
    background: rgba(155, 89, 208, 0.1);
    border-color: #9b59d0;
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }

  .feature-id {
    color: #9b59d0;
    font-size: 11px;
    font-weight: 600;
    font-family: monospace;
  }

  .priority-badge {
    color: #fff;
    padding: 1px 6px;
    border-radius: 8px;
    font-size: 9px;
    font-weight: 600;
  }

  .feature-title {
    font-size: 13px;
    font-weight: 600;
    color: #fff;
    margin-bottom: 4px;
    line-height: 1.3;
  }

  .feature-description {
    font-size: 11px;
    color: #888;
    line-height: 1.4;
    margin-bottom: 8px;
  }

  .feature-meta {
    margin-bottom: 8px;
  }

  .feature-progress {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
  }

  .progress-bar {
    flex: 1;
    height: 4px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 2px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    transition: width 0.3s ease;
  }

  .progress-text {
    font-size: 10px;
    color: #888;
    font-weight: 600;
    min-width: 30px;
  }

  .feature-tags {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }

  .tag {
    background: rgba(155, 89, 208, 0.2);
    color: #9b59d0;
    padding: 1px 4px;
    border-radius: 8px;
    font-size: 9px;
    font-weight: 500;
  }

  .tag-more {
    color: #666;
    font-size: 9px;
  }

  .dependencies {
    font-size: 10px;
    color: #666;
    border-top: 1px solid #333;
    padding-top: 6px;
    margin-top: 6px;
  }

  .dep-label {
    margin-right: 4px;
  }

  .dependency {
    color: #f59e0b;
    margin-right: 4px;
  }

  .dep-more {
    color: #666;
  }

  .dependencies-view {
    background: linear-gradient(135deg, #1a1a1a 0%, #2a2a2a 100%);
    border: 1px solid #333;
    border-radius: 12px;
    padding: 20px;
  }

  .graph-container {
    width: 100%;
  }

  .graph-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }

  .graph-header h3 {
    color: #9b59d0;
    margin: 0;
    font-size: 18px;
  }

  .graph-controls {
    display: flex;
    gap: 8px;
  }

  .graph-btn {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid #333;
    color: #888;
    padding: 6px 12px;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }

  .dependency-graph {
    background: rgba(0, 0, 0, 0.3);
    border-radius: 8px;
    overflow: hidden;
  }

  .timeline-view {
    background: linear-gradient(135deg, #1a1a1a 0%, #2a2a2a 100%);
    border: 1px solid #333;
    border-radius: 12px;
    padding: 20px;
  }

  .velocity-chart h3 {
    color: #9b59d0;
    margin: 0 0 20px 0;
    font-size: 18px;
  }

  .chart-container {
    display: flex;
    gap: 12px;
    align-items: end;
    height: 120px;
    margin-bottom: 20px;
  }

  .velocity-bar {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    height: 100%;
  }

  .bar-fill {
    background: linear-gradient(to top, #9b59d0, #c084fc);
    border-radius: 4px 4px 0 0;
    width: 100%;
    min-height: 4px;
    transition: height 0.5s ease;
  }

  .bar-label {
    color: #888;
    font-size: 11px;
    margin-top: 8px;
  }

  .bar-value {
    color: #9b59d0;
    font-size: 12px;
    font-weight: 600;
    margin-top: 2px;
  }

  .completion-prediction {
    background: rgba(155, 89, 208, 0.1);
    border: 1px solid rgba(155, 89, 208, 0.3);
    border-radius: 8px;
    padding: 16px;
  }

  .prediction-card {
    text-align: center;
  }

  .prediction-title {
    color: #888;
    font-size: 12px;
    margin-bottom: 8px;
  }

  .prediction-date {
    color: #9b59d0;
    font-size: 24px;
    font-weight: 700;
    margin-bottom: 4px;
  }

  .prediction-confidence {
    color: #888;
    font-size: 11px;
  }

  .dialog-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .dialog {
    background: linear-gradient(135deg, #1a1a1a 0%, #2a2a2a 100%);
    border: 1px solid #333;
    border-radius: 12px;
    width: 500px;
    max-width: 90vw;
    max-height: 80vh;
    overflow: hidden;
  }

  .dialog-header {
    padding: 16px 20px;
    border-bottom: 1px solid #333;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .dialog-header h3 {
    color: #9b59d0;
    margin: 0;
    font-size: 16px;
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
  }

  .dialog-content {
    padding: 20px;
  }

  .form-group {
    margin-bottom: 16px;
  }

  .form-group label {
    display: block;
    color: #888;
    font-size: 12px;
    margin-bottom: 6px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .form-group input,
  .form-group textarea,
  .form-group select {
    width: 100%;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid #333;
    color: #fff;
    padding: 8px 12px;
    border-radius: 4px;
    font-size: 14px;
  }

  .form-group textarea {
    resize: vertical;
    min-height: 80px;
  }

  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }

  .dialog-actions {
    padding: 16px 20px;
    border-top: 1px solid #333;
    display: flex;
    justify-content: flex-end;
    gap: 12px;
  }

  .cancel-btn {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid #333;
    color: #888;
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
  }

  /* F0229: Feature State Legend Styles */
  .feature-legend {
    background: linear-gradient(135deg, #1a1a1a 0%, #2a2a2a 100%);
    border: 1px solid #333;
    border-radius: 12px;
    padding: 20px;
    margin-bottom: 24px;
  }

  .feature-legend h3 {
    color: #9b59d0;
    margin: 0 0 16px 0;
    font-size: 18px;
    font-weight: 600;
  }

  .legend-items {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 12px;
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px;
    background: rgba(255, 255, 255, 0.03);
    border-radius: 6px;
    transition: background 0.2s ease;
  }

  .legend-item:hover {
    background: rgba(255, 255, 255, 0.06);
  }

  .legend-icon {
    font-size: 16px;
    font-weight: 600;
    min-width: 20px;
    text-align: center;
  }

  .legend-info {
    flex: 1;
  }

  .legend-title {
    color: #fff;
    font-size: 12px;
    font-weight: 600;
    margin-bottom: 2px;
  }

  .legend-desc {
    color: #888;
    font-size: 11px;
    line-height: 1.3;
  }

  /* F0230: Enhanced Feature Card Details Styles */
  .feature-details {
    margin: 8px 0;
    padding: 8px 0;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .detail-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 4px;
  }

  .detail-row:last-child {
    margin-bottom: 0;
  }

  .detail-item {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 10px;
  }

  .detail-label {
    color: #666;
    font-weight: 500;
  }

  .detail-value {
    color: #aaa;
    font-weight: 600;
  }

  .category-badge {
    background: rgba(59, 130, 246, 0.2);
    color: #60a5fa;
    padding: 2px 6px;
    border-radius: 8px;
    font-size: 9px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  /* F0231: Enhanced Dependencies Graph Styles */
  .feature-node {
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .feature-node:hover .node-circle {
    stroke: #9b59d0;
    stroke-width: 3;
    filter: brightness(1.2);
  }

  .dependency-line {
    transition: all 0.2s ease;
  }

  .dependency-line:hover {
    stroke: #9b59d0;
    stroke-width: 3;
  }

  .graph-legend {
    position: absolute;
    top: 10px;
    right: 10px;
    background: rgba(26, 26, 26, 0.9);
    border: 1px solid #333;
    border-radius: 6px;
    padding: 12px;
    font-size: 10px;
  }

  .legend-row {
    display: flex;
    gap: 16px;
    margin-bottom: 8px;
  }

  .legend-row:last-child {
    margin-bottom: 0;
  }

  .legend-item-small {
    display: flex;
    align-items: center;
    gap: 6px;
    color: #aaa;
  }

  .legend-circle {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .legend-line {
    width: 16px;
    height: 2px;
    background: #666;
    border-radius: 1px;
  }

  .legend-dot {
    width: 6px;
    height: 6px;
    background: #f59e0b;
    border-radius: 50%;
  }

  .dependency-graph {
    position: relative;
    background: rgba(0, 0, 0, 0.3);
    border-radius: 8px;
    overflow: hidden;
  }
</style>