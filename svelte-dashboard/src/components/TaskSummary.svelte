<script>
  import { taskMetrics, tasksStore } from '../stores.js';
  
  $: metrics = $taskMetrics;
  $: tasks = $tasksStore;

  function getTaskIcon(status) {
    switch (status) {
      case 'pending': return '⏳';
      case 'in_progress': return '🔄';
      case 'completed': return '✅';
      case 'blocked': return '🚫';
      default: return '❓';
    }
  }

  function getTaskColor(status) {
    switch (status) {
      case 'pending': return '#888';
      case 'in_progress': return '#f59e0b';
      case 'completed': return '#4ade80';
      case 'blocked': return '#ef4444';
      default: return '#666';
    }
  }

  $: recentTasks = $tasks.slice(0, 5);
</script>

<div class="task-summary-card">
  <h2>Task Summary</h2>
  
  <div class="task-stats">
    <div class="stat-item pending">
      <div class="stat-icon">⏳</div>
      <div class="stat-info">
        <div class="stat-count">{metrics.pending}</div>
        <div class="stat-label">Pending</div>
      </div>
    </div>
    
    <div class="stat-item progress">
      <div class="stat-icon">🔄</div>
      <div class="stat-info">
        <div class="stat-count">{metrics.inProgress}</div>
        <div class="stat-label">In Progress</div>
      </div>
    </div>
    
    <div class="stat-item completed">
      <div class="stat-icon">✅</div>
      <div class="stat-info">
        <div class="stat-count">{metrics.completed}</div>
        <div class="stat-label">Completed</div>
      </div>
    </div>
    
    <div class="stat-item blocked">
      <div class="stat-icon">🚫</div>
      <div class="stat-info">
        <div class="stat-count">{metrics.blocked}</div>
        <div class="stat-label">Blocked</div>
      </div>
    </div>
  </div>

  <div class="active-tasks">
    <h3>Active Tasks</h3>
    <div class="task-list">
      {#each recentTasks as task}
        <div class="task-item" style="border-left-color: {getTaskColor(task.status)}">
          <div class="task-header">
            <span class="task-icon">{getTaskIcon(task.status)}</span>
            <span class="task-id">{task.id}</span>
            <span class="task-status" style="color: {getTaskColor(task.status)}">{task.status}</span>
          </div>
          <div class="task-content">{task.content || task.description}</div>
        </div>
      {:else}
        <div class="task-placeholder">
          <div class="task-header">
            <span class="task-icon">🔄</span>
            <span class="task-id">todo-001</span>
            <span class="task-status" style="color: #f59e0b">in_progress</span>
          </div>
          <div class="task-content">Set up Svelte framework for ADE implementation</div>
        </div>
        <div class="task-placeholder">
          <div class="task-header">
            <span class="task-icon">⏳</span>
            <span class="task-id">todo-002</span>
            <span class="task-status" style="color: #888">pending</span>
          </div>
          <div class="task-content">Implement ADE Overview features</div>
        </div>
        <div class="task-placeholder">
          <div class="task-header">
            <span class="task-icon">⏳</span>
            <span class="task-id">todo-003</span>
            <span class="task-status" style="color: #888">pending</span>
          </div>
          <div class="task-content">Create Svelte components for entity management</div>
        </div>
      {/each}
    </div>
  </div>
</div>

