<script>
  import { websocketConnected, agentActivity } from '../stores.js';
  
  export let currentView;

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

  function setView(viewId) {
    currentView = viewId;
  }
</script>

<header class="ade-header">
  <div class="header-content">
    <div class="brand">
      <h1>ADE Workspace</h1>
      <div class="status-indicators">
        <div class="status-item">
          <span class="status-dot" class:online={$websocketConnected}></span>
          <span class="status-label">{$websocketConnected ? 'Connected' : 'Disconnected'}</span>
        </div>
      </div>
    </div>

    <nav class="main-nav">
      {#each navItems as item}
        <button 
          class="nav-item" 
          class:active={currentView === item.id}
          on:click={() => setView(item.id)}
        >
          {item.label}
        </button>
      {/each}
    </nav>

    <div class="agent-status">
      <div class="agent-info">
        <div class="agent-model">{$agentActivity.model}</div>
        <div class="agent-task">{$agentActivity.currentTask}</div>
      </div>
      <div class="context-usage">
        <div class="context-label">Context</div>
        <div class="context-bar">
          <div class="context-fill" style="width: {$agentActivity.contextUsage}%"></div>
        </div>
        <div class="context-percent">{$agentActivity.contextUsage}%</div>
      </div>
    </div>
  </div>
</header>

<style>
  .ade-header {
    background: #ffffff;
    border-bottom: 1px solid #e5e7eb;
    padding: 0 24px;
    box-shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px 0 rgba(0, 0, 0, 0.06);
  }

  .header-content {
    max-width: 1400px;
    margin: 0 auto;
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 70px;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 20px;
  }

  .brand h1 {
    color: #111827;
    margin: 0;
    font-size: 20px;
    font-weight: 600;
    letter-spacing: -0.025em;
  }

  .status-indicators {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .status-item {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: #6b7280;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #666;
    transition: background 0.3s ease;
  }

  .status-dot.online {
    background: #10b981;
  }

  .main-nav {
    display: flex;
    gap: 2px;
  }

  .nav-item {
    padding: 8px 16px;
    background: none;
    border: none;
    color: #6b7280;
    cursor: pointer;
    border-radius: 6px;
    transition: all 0.15s ease;
    font-size: 14px;
    font-weight: 500;
    letter-spacing: -0.025em;
  }

  .nav-item:hover {
    background: #f3f4f6;
    color: #374151;
  }

  .nav-item.active {
    background: #dbeafe;
    color: #1e40af;
  }

  .agent-status {
    display: flex;
    flex-direction: column;
    gap: 8px;
    align-items: flex-end;
  }

  .agent-info {
    text-align: right;
    font-size: 12px;
  }

  .agent-model {
    color: #1f2937;
    font-weight: 600;
  }

  .agent-task {
    color: #6b7280;
    max-width: 200px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .context-usage {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 11px;
  }

  .context-label {
    color: #6b7280;
  }

  .context-bar {
    width: 60px;
    height: 4px;
    background: #e5e7eb;
    border-radius: 2px;
    overflow: hidden;
  }

  .context-fill {
    height: 100%;
    background: linear-gradient(90deg, #10b981, #f59e0b, #ef4444);
    transition: width 0.3s ease;
  }

  .context-percent {
    color: #6b7280;
    min-width: 30px;
    font-weight: 500;
  }
</style>