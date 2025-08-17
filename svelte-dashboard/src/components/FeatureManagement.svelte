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
      state: 'complete',
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
      state: 'implemented',
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
      state: 'tested',
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
      state: 'pending',
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
      state: 'implemented',
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
      state: 'issues',
      category: 'Core',
      priority: 'high',
      dependencies: [],
      assignee: 'Claude',
      estimatedHours: 3,
      completedHours: 3,
      tags: ['filtering', 'categories', 'ui']
    }
  ];

  $: features = ($featuresStore && Array.isArray($featuresStore) && $featuresStore.length > 0) ? $featuresStore : sampleFeatures;
  $: filteredFeatures = Array.isArray(features) ? features.filter(feature => 
    filterCategory === 'all' || feature.category === filterCategory
  ) : [];
  $: featuresByState = {
    'pending': (filteredFeatures || []).filter(f => f.state === 'pending'),
    'implemented': (filteredFeatures || []).filter(f => f.state === 'implemented'), 
    'tested': (filteredFeatures || []).filter(f => f.state === 'tested'),
    'complete': (filteredFeatures || []).filter(f => f.state === 'complete'),
    'issues': (filteredFeatures || []).filter(f => f.state === 'issues'),
    'blocked': (filteredFeatures || []).filter(f => f.state === 'blocked')
  };
  $: categories = [...new Set(features.map(f => f.category))];
  $: velocityData = calculateVelocity();

  function getStateLabel(state) {
    switch (state) {
      case 'pending': return 'Not Started';
      case 'implemented': return 'In Progress';
      case 'tested': return 'Testing';
      case 'complete': return 'Complete';
      case 'issues': return 'Issues';
      case 'blocked': return 'Blocked';
      default: return 'Unknown';
    }
  }

  function getStateColor(state) {
    switch (state) {
      case 'pending': return 'var(--color-text-secondary)';
      case 'implemented': return 'var(--color-info)';
      case 'tested': return 'var(--color-warning)';
      case 'complete': return 'var(--color-success)';
      case 'issues': return 'var(--color-error)';
      case 'blocked': return 'var(--color-error)';
      default: return 'var(--color-text-secondary)';
    }
  }

  function getPriorityColor(priority) {
    switch (priority) {
      case 'high': return 'var(--color-error)';
      case 'medium': return 'var(--color-warning)';
      case 'low': return 'var(--color-success)';
      default: return 'var(--color-text-secondary)';
    }
  }

  function selectFeature(feature) {
    selectedFeature = feature;
  }

  function calculateVelocity() {
    // Calculate weekly velocity based on completed features
    const completedFeatures = features.filter(f => f.state === 'complete');
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

<div class="feature-management-container">
  <div class="feature-header">
    <h1 class="text-primary">Feature Management</h1>
    
    <div class="header-controls">
      <div class="view-toggle">
        <button 
          class="btn-secondary toggle-btn" 
          class:active={viewMode === 'kanban'}
          on:click={() => viewMode = 'kanban'}
        >
          Kanban
        </button>
        <button 
          class="btn-secondary toggle-btn" 
          class:active={viewMode === 'timeline'}
          on:click={() => viewMode = 'timeline'}
        >
          Timeline
        </button>
        <button 
          class="btn-secondary toggle-btn" 
          class:active={viewMode === 'dependencies'}
          on:click={() => viewMode = 'dependencies'}
        >
          Dependencies
        </button>
      </div>
      
      <div class="header-filters">
        <select bind:value={filterCategory} class="category-filter bg-surface border rounded-md">
          <option value="all">All Categories</option>
          {#each categories as category}
            <option value={category}>{category}</option>
          {/each}
        </select>
        
        <button class="btn-primary create-btn" on:click={createFeature}>
          + Add Feature
        </button>
      </div>
    </div>
  </div>

  <div class="feature-stats bg-surface-2 rounded-lg">
    <div class="stat-card card bg-surface">
      <div class="stat-number text-primary">{filteredFeatures.length}</div>
      <div class="stat-label text-secondary">Total Features</div>
    </div>
    <div class="stat-card card bg-surface complete">
      <div class="stat-number text-success">{featuresByState['complete'].length}</div>
      <div class="stat-label text-secondary">Complete</div>
    </div>
    <div class="stat-card card bg-surface progress">
      <div class="stat-number text-info">{featuresByState['implemented'].length + featuresByState['tested'].length}</div>
      <div class="stat-label text-secondary">In Progress</div>
    </div>
    <div class="stat-card card bg-surface blocked">
      <div class="stat-number text-error">{featuresByState['issues'].length + featuresByState['blocked'].length}</div>
      <div class="stat-label text-secondary">Issues</div>
    </div>
    <div class="stat-card card bg-surface velocity">
      <div class="stat-number text-primary">17.2</div>
      <div class="stat-label text-secondary">Features/Week</div>
    </div>
  </div>

  <!-- F0229: Feature State Legend -->
  <div class="feature-legend card bg-surface-2 rounded-lg">
    <h3 class="text-primary">Feature State Legend</h3>
    <div class="legend-items">
      <div class="legend-item">
        <div class="legend-icon state-pending"></div>
        <div class="legend-info">
          <div class="legend-title text-primary">Not Started</div>
          <div class="legend-desc text-secondary">Feature not yet implemented</div>
        </div>
      </div>
      <div class="legend-item">
        <div class="legend-icon state-implemented"></div>
        <div class="legend-info">
          <div class="legend-title text-primary">In Progress</div>
          <div class="legend-desc text-secondary">Implementation in code but lacks dedicated tests</div>
        </div>
      </div>
      <div class="legend-item">
        <div class="legend-icon state-tested"></div>
        <div class="legend-info">
          <div class="legend-title text-primary">Testing</div>
          <div class="legend-desc text-secondary">Implemented with dedicated FAILING tests</div>
        </div>
      </div>
      <div class="legend-item">
        <div class="legend-icon state-complete"></div>
        <div class="legend-info">
          <div class="legend-title text-primary">Complete</div>
          <div class="legend-desc text-secondary">Implemented with dedicated PASSING tests</div>
        </div>
      </div>
      <div class="legend-item">
        <div class="legend-icon state-issues"></div>
        <div class="legend-info">
          <div class="legend-title text-primary">Issues</div>
          <div class="legend-desc text-secondary">Has tests but they are fake/tautological/broken</div>
        </div>
      </div>
      <div class="legend-item">
        <div class="legend-icon state-blocked"></div>
        <div class="legend-info">
          <div class="legend-title text-primary">Critical Issue</div>
          <div class="legend-desc text-secondary">Critical issue requiring immediate attention</div>
        </div>
      </div>
    </div>
  </div>

  {#if viewMode === 'kanban'}
    <div class="kanban-board bg-surface-3 rounded-lg">
      {#each Object.entries(featuresByState) as [state, stateFeatures]}
        <div 
          class="kanban-column card bg-surface"
          on:dragover={handleDragOver}
          on:drop={(e) => handleDrop(e, state)}
        >
          <div class="column-header state-{state}">
            <div class="column-title">
              <div class="state-indicator state-{state}"></div>
              {getStateLabel(state)}
            </div>
            <div class="column-count">{stateFeatures.length}</div>
          </div>
          
          <div class="column-content">
            {#each stateFeatures as feature}
              <div 
                class="feature-card card bg-surface-2 border"
                class:selected={selectedFeature?.id === feature.id}
                draggable="true"
                on:dragstart={(e) => handleDragStart(e, feature)}
                on:click={() => selectFeature(feature)}
              >
                <div class="card-header">
                  <div class="feature-id text-secondary">{feature.id}</div>
                  <div 
                    class="priority-badge priority-{feature.priority}"
                  >
                    {feature.priority.toUpperCase()}
                  </div>
                </div>
                
                <div class="feature-title text-primary">{feature.title}</div>
                <div class="feature-description text-secondary">{feature.description}</div>
                
                <div class="feature-meta">
                  <div class="feature-progress">
                    <div class="progress-bar">
                      <div 
                        class="progress-fill state-{feature.state}" 
                        style:width="{getCompletionPercentage(feature)}%"
                      ></div>
                    </div>
                    <span class="progress-text">{getCompletionPercentage(feature)}%</span>
                  </div>
                  
                  <!-- F0230: Enhanced Card Details -->
                  <div class="feature-details">
                    <div class="detail-row">
                      <div class="detail-item">
                        <span class="detail-label text-secondary">Effort:</span>
                        <span class="detail-value text-primary">{feature.completedHours}h / {feature.estimatedHours}h</span>
                      </div>
                      {#if feature.assignee}
                        <div class="detail-item">
                          <span class="detail-label text-secondary">Assignee:</span>
                          <span class="detail-value text-primary">{feature.assignee}</span>
                        </div>
                      {/if}
                    </div>
                    {#if feature.category}
                      <div class="detail-row">
                        <div class="category-badge bg-surface-3 text-secondary rounded-md">{feature.category}</div>
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
    <div class="dependencies-view card bg-surface rounded-lg">
      <div class="graph-container">
        <div class="graph-header">
          <h3>Feature Dependencies Graph</h3>
          <div class="graph-controls">
            <button class="btn-secondary graph-btn">Reset View</button>
            <button class="btn-secondary graph-btn">Auto Layout</button>
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
                  fill="var(--color-text-secondary)" 
                  class="node-circle state-{feature.state}" 
                  class:selected={selectedFeature?.id === feature.id}
                />
                
                <!-- Feature ID -->
                <text
                  x="0"
                  y="-5"
                  text-anchor="middle"
                  fill="var(--color-text-inverse)"
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
                  fill="var(--color-text-secondary)" 
                  class="priority-fill priority-{feature.priority}"
                  stroke="var(--color-border)"
                  stroke-width="1"
                />
                
                <!-- Feature title -->
                <text
                  x="0"
                  y="50"
                  text-anchor="middle"
                  fill="var(--color-text-tertiary)"
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
                  fill="var(--color-bg-surface-3)"
                  rx="2"
                />
                <rect
                  x="-25"
                  y="20"
                  width="{getCompletionPercentage(feature) / 2}"
                  height="4"
                  fill="var(--color-text-secondary)" 
                  class="progress-fill state-{feature.state}"
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
                      stroke="var(--color-border)"
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
                      fill="var(--color-warning)"
                      stroke="var(--color-border)"
                      stroke-width="1"
                    />
                  {/if}
                {/each}
              {/each}
            {/each}
            
            <defs>
              <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
                <polygon points="0 0, 10 3.5, 0 7" fill="var(--color-border)" />
              </marker>
            </defs>
          </svg>
          
          <!-- Graph legend -->
          <div class="graph-legend">
            <div class="legend-row">
              <div class="legend-item-small">
                <div class="legend-circle"></div>
                <span>High Priority</span>
              </div>
              <div class="legend-item-small">
                <div class="legend-circle"></div>
                <span>Medium Priority</span>
              </div>
              <div class="legend-item-small">
                <div class="legend-circle"></div>
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
    <div class="timeline-view card bg-surface rounded-lg">
      <div class="velocity-chart">
        <h3 class="text-primary">Implementation Velocity</h3>
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
            <div class="prediction-title text-secondary">Estimated Completion</div>
            <div class="prediction-date text-primary">2 weeks</div>
            <div class="prediction-confidence text-secondary">85% confidence</div>
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>

<!-- Feature Creation Dialog -->
{#if showCreateDialog}
  <div class="dialog-overlay bg-surface-3" on:click={closeCreateDialog}>
    <div class="dialog card bg-surface rounded-lg" on:click|stopPropagation>
      <div class="dialog-header">
        <h3 class="text-primary">Add New Feature</h3>
        <button class="close-btn text-secondary" on:click={closeCreateDialog}>×</button>
      </div>
      
      <div class="dialog-content">
        <div class="form-group">
          <label class="text-secondary">Feature Title</label>
          <input type="text" class="bg-surface border rounded-md" placeholder="Enter feature title..." />
        </div>
        
        <div class="form-group">
          <label class="text-secondary">Description</label>
          <textarea class="bg-surface border rounded-md" placeholder="Describe the feature..."></textarea>
        </div>
        
        <div class="form-row">
          <div class="form-group">
            <label class="text-secondary">Category</label>
            <select class="bg-surface border rounded-md">
              <option value="ADE Interface">ADE Interface</option>
              <option value="Core">Core</option>
              <option value="Analytics">Analytics</option>
            </select>
          </div>
          
          <div class="form-group">
            <label class="text-secondary">Priority</label>
            <select class="bg-surface border rounded-md">
              <option value="high">High</option>
              <option value="medium">Medium</option>
              <option value="low">Low</option>
            </select>
          </div>
        </div>
        
        <div class="form-group">
          <label class="text-secondary">Estimated Hours</label>
          <input type="number" class="bg-surface border rounded-md" min="1" max="100" value="8" />
        </div>
      </div>
      
      <div class="dialog-actions">
        <button class="btn-secondary cancel-btn" on:click={closeCreateDialog}>Cancel</button>
        <button class="btn-primary create-btn">Create Feature</button>
      </div>
    </div>
  </div>
{/if}

