<script>
  import { onMount } from 'svelte';
  import { tasksStore } from '../stores.js';

  let tasks = [];
  let draggedTask = null;
  
  const taskColumns = [
    { id: 'pending', title: 'To Do', color: '#6b7280', emoji: '📋' },
    { id: 'in_progress', title: 'In Progress', color: '#f59e0b', emoji: '🔄' },
    { id: 'review', title: 'Review', color: '#8b5cf6', emoji: '👀' },
    { id: 'completed', title: 'Completed', color: '#10b981', emoji: '✅' },
    { id: 'blocked', title: 'Blocked', color: '#ef4444', emoji: '🚫' }
  ];

  onMount(() => {
    // Load tasks from store or use sample data
    const sampleTasks = [
      { id: 'task-001', title: 'Implement Task Kanban Board', description: 'Create drag-and-drop task management interface', status: 'in_progress', priority: 'high', assignee: 'Claude', effort: '4h' },
      { id: 'task-002', title: 'Add Task State Management', description: 'Implement task status transitions and validation', status: 'pending', priority: 'medium', assignee: 'Claude', effort: '2h' },
      { id: 'task-003', title: 'Create Task Detail Modal', description: 'Modal dialog for editing task properties', status: 'pending', priority: 'low', assignee: 'Claude', effort: '3h' },
      { id: 'task-004', title: 'Task Search and Filtering', description: 'Add search functionality and status filters', status: 'review', priority: 'medium', assignee: 'Claude', effort: '2h' },
      { id: 'task-005', title: 'Task API Integration', description: 'Connect kanban board to backend API', status: 'completed', priority: 'high', assignee: 'Claude', effort: '6h' }
    ];
    
    tasks = $tasksStore.length > 0 ? $tasksStore : sampleTasks;
  });

  function getTasksByStatus(status) {
    return tasks.filter(task => task.status === status);
  }

  function getPriorityColor(priority) {
    switch (priority) {
      case 'high': return '#ef4444';
      case 'medium': return '#f59e0b';
      case 'low': return '#10b981';
      default: return '#6b7280';
    }
  }

  function handleDragStart(event, task) {
    draggedTask = task;
    event.dataTransfer.effectAllowed = 'move';
    event.target.classList.add('dragging');
  }

  function handleDragEnd(event) {
    event.target.classList.remove('dragging');
    draggedTask = null;
  }

  function handleDragOver(event) {
    event.preventDefault();
    event.dataTransfer.dropEffect = 'move';
  }

  function handleDrop(event, newStatus) {
    event.preventDefault();
    if (draggedTask && draggedTask.status !== newStatus) {
      // Update task status
      tasks = tasks.map(task => 
        task.id === draggedTask.id 
          ? { ...task, status: newStatus }
          : task
      );
      
      // Update store
      tasksStore.set(tasks);
      
      // TODO: Call API to update task status
      console.log(`Task ${draggedTask.id} moved to ${newStatus}`);
    }
  }

  function addNewTask(status) {
    const newTask = {
      id: `task-${Date.now()}`,
      title: 'New Task',
      description: 'Task description',
      status: status,
      priority: 'medium',
      assignee: 'Unassigned',
      effort: '1h'
    };
    
    tasks = [...tasks, newTask];
    tasksStore.set(tasks);
  }
</script>

<div class="task-kanban">
  <div class="kanban-header">
    <h2>Task Management</h2>
    <div class="kanban-actions">
      <button class="btn-primary" on:click={() => addNewTask('pending')}>
        + Add Task
      </button>
    </div>
  </div>

  <div class="kanban-board">
    {#each taskColumns as column}
      <div 
        class="kanban-column"
        on:dragover={handleDragOver}
        on:drop={(e) => handleDrop(e, column.id)}
      >
        <div class="column-header" style="border-color: {column.color}">
          <span class="column-emoji">{column.emoji}</span>
          <span class="column-title">{column.title}</span>
          <span class="column-count">{getTasksByStatus(column.id).length}</span>
        </div>
        
        <div class="column-tasks">
          {#each getTasksByStatus(column.id) as task}
            <div 
              class="task-card" 
              draggable="true"
              on:dragstart={(e) => handleDragStart(e, task)}
              on:dragend={handleDragEnd}
            >
              <div class="task-header">
                <div class="task-id">{task.id}</div>
                <div class="task-priority" style="background-color: {getPriorityColor(task.priority)}">
                  {task.priority}
                </div>
              </div>
              
              <div class="task-title">{task.title}</div>
              <div class="task-description">{task.description}</div>
              
              <div class="task-footer">
                <div class="task-assignee">👤 {task.assignee}</div>
                <div class="task-effort">⏱️ {task.effort}</div>
              </div>
            </div>
          {/each}
          
          {#if getTasksByStatus(column.id).length === 0}
            <div class="empty-column">
              <div class="empty-message">No tasks</div>
              <button 
                class="empty-add-btn"
                on:click={() => addNewTask(column.id)}
              >
                + Add task
              </button>
            </div>
          {/if}
        </div>
      </div>
    {/each}
  </div>
</div>

