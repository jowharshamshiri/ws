<script>
  import { projectStore, agentActivity } from '../stores.js';
  
  $: project = $projectStore;
  $: activity = $agentActivity;
</script>

<div class="overview-container">
  <!-- Project Status Card - Full card for complex content -->
  <div class="card card--project card--span-2">
    <div class="card__header">
      <h3 class="card__title">Project Status</h3>
      <div class="badge badge--success">Active</div>
    </div>
    <div class="project-info">
      <div class="project-name">{project.name}</div>
      <div class="project-description">{project.description}</div>
      <div class="project-version">v{project.version}</div>
    </div>
  </div>

  <!-- Compact Stats Grid - Appwrite style small metrics -->
  <div class="stats-grid">
    <!-- Implementation Progress Stat -->
    <div class="stat-box">
      <div class="stat-header">
        <div class="stat-title">Implementation Progress</div>
        <div class="stat-indicator success">56%</div>
      </div>
      <div class="stat-content">
        <span class="metric-number">179</span>
        <span class="metric-label">Implemented</span>
      </div>
      <div class="stat-progress">
        <div class="progress-bar">
          <div class="progress-fill" style="width: 59.7%"></div>
        </div>
        <div class="progress-text">121 remaining</div>
      </div>
    </div>

    <!-- Test Coverage Stat -->
    <div class="stat-box">
      <div class="stat-header">
        <div class="stat-title">Test Coverage</div>
        <div class="stat-indicator success">55%</div>
      </div>
      <div class="stat-content">
        <span class="metric-number">163</span>
        <span class="metric-label">Tested</span>
      </div>
      <div class="stat-progress">
        <div class="progress-bar">
          <div class="progress-fill success" style="width: 54.3%"></div>
        </div>
        <div class="progress-text">137 pending</div>
      </div>
    </div>

    <!-- Development Velocity Stat -->
    <div class="stat-box">
      <div class="stat-header">
        <div class="stat-title">Development Velocity</div>
        <div class="stat-indicator success">↗ +14.3</div>
      </div>
      <div class="stat-content">
        <span class="metric-number">14.3</span>
        <span class="metric-label">features/week</span>
      </div>
      <div class="stat-trend">
        <span class="trend-icon up">↗</span>
        <span class="trend-value up">+14.3</span>
      </div>
    </div>

    <!-- Active Issues Stat -->
    <div class="stat-box">
      <div class="stat-header">
        <div class="stat-title">Active Issues</div>
        <div class="stat-indicator info">0</div>
      </div>
      <div class="stat-content">
        <span class="metric-number">0</span>
        <span class="metric-label">Issues</span>
      </div>
    </div>
  </div>

  <!-- Active Session Card -->
  <div class="card card--session card--span-2">
    <div class="card__header">
      <h3 class="card__title">Active Session</h3>
      <div class="badge badge--success">Online</div>
    </div>
    <div class="session-content">
      <div class="session-info">
        <div class="session-item">
          <span class="label">AI Agent:</span>
          <span class="value">{activity.model}</span>
        </div>
        <div class="session-item">
          <span class="label">Current Task:</span>
          <span class="value">{activity.currentTask}</span>
        </div>
        <div class="session-item">
          <span class="label">Duration:</span>
          <span class="value">{activity.sessionDuration}</span>
        </div>
        <div class="session-item">
          <span class="label">Context Usage:</span>
          <div class="context-usage">
            <div class="progress-bar progress-bar--small">
              <div class="progress-bar__fill" style="width: {activity.contextUsage}%"></div>
            </div>
            <span class="context-percent" class:warning={activity.contextUsage > 80}>
              {activity.contextUsage}%
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- Recent Activity Card -->
  <div class="card card--activity card--span-2">
    <div class="card__header">
      <h3 class="card__title">Recent Activity</h3>
      <div class="badge badge--secondary">3 actions</div>
    </div>
    <div class="activity-content">
      <div class="timeline">
        {#each activity.recentActions as action}
          <div class="timeline-item">
            <div class="timeline-time">{action.time}</div>
            <div class="timeline-description">{action.description}</div>
            <div class="timeline-type badge badge--small">{action.type || 'System'}</div>
          </div>
        {:else}
          <div class="timeline-item">
            <div class="timeline-time">12:36</div>
            <div class="timeline-description">ADE overview dashboard implemented</div>
            <div class="timeline-type badge badge--small">Feature</div>
          </div>
          <div class="timeline-item">
            <div class="timeline-time">12:35</div>
            <div class="timeline-description">Svelte components analyzed</div>
            <div class="timeline-type badge badge--small">Analysis</div>
          </div>
          <div class="timeline-item">
            <div class="timeline-time">12:34</div>
            <div class="timeline-description">Session initialized successfully</div>
            <div class="timeline-type badge badge--small">System</div>
          </div>
        {/each}
      </div>
    </div>
  </div>
</div>

