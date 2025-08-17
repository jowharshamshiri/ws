<script>
  import { agentActivity } from '../stores.js';
  
  // Props from parent App component
  export let currentView;
  export let switchView;
  
  const navItems = [
    { id: 'overview', label: 'Overview' },
    { id: 'sessions', label: 'Sessions' },
    { id: 'issues', label: 'Issues' },
    { id: 'features', label: 'Features' },
    { id: 'workspace', label: 'Workspace' },
    { id: 'testing', label: 'Testing' },
    { id: 'entities', label: 'Entities' },
    { id: 'analytics', label: 'Analytics' },
    { id: 'settings', label: 'Settings' }
  ];
</script>

<header>
  <div class="header-content">
    <div class="brand">
      <h1>Workspace</h1>
    </div>
    
    <nav>
      {#each navItems as item}
        <button 
          class="nav-item" 
          class:active={currentView === item.id}
          on:click={() => switchView(item.id)}
        >
          {item.label}
        </button>
      {/each}
    </nav>
    
    <div class="status">
      <span>{$agentActivity.model}</span>
      <span>Context: {$agentActivity.contextUsage}%</span>
    </div>
  </div>
</header>
