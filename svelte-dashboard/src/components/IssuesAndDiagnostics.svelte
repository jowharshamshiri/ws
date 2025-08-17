<script>
  import { onMount } from 'svelte';
  import { issuesStore } from '../stores.js';
  
  let issues = [];
  let selectedIssue = null;
  let filterEnvironment = 'all';
  let filterSeverity = 'all';
  let sortBy = 'timestamp';
  
  // Sample issue data
  const sampleIssues = [
    {
      id: 'issue-001',
      title: 'Feature validation timeout in F0210 implementation',
      severity: 'critical',
      environment: 'development',
      timestamp: '2025-08-08T23:15:32Z',
      description: 'SessionReplay component validation failing with timeout after 30 seconds',
      stackTrace: 'Error: Timeout waiting for session data\n  at SessionReplay.loadSession (line 42)\n  at SessionReplay.selectSession (line 51)',
      aiAnalysis: {
        rootCause: 'API endpoint /api/sessions is not responding within expected timeout',
        suggestions: [
          'Check if backend MCP server is running',
          'Increase timeout threshold for session loading',
          'Add retry logic with exponential backoff',
          'Implement graceful fallback to sample data'
        ],
        confidence: 0.85
      },
      responseTime: 3200,
      tags: ['timeout', 'api', 'session-replay'],
      resolved: false
    },
    {
      id: 'issue-002', 
      title: 'Memory leak in WebSocket connection handling',
      severity: 'high',
      environment: 'production',
      timestamp: '2025-08-08T22:45:18Z',
      description: 'WebSocket connections not being properly closed causing memory usage to increase over time',
      stackTrace: 'Warning: WebSocket connection not cleaned up\n  at WebSocketService.disconnect (line 78)\n  at component cleanup',
      aiAnalysis: {
        rootCause: 'WebSocket cleanup not triggered in component unmount lifecycle',
        suggestions: [
          'Add onDestroy cleanup for WebSocket connections',
          'Implement connection pooling with automatic cleanup',
          'Add memory monitoring and alerting',
          'Review component lifecycle management'
        ],
        confidence: 0.92
      },
      responseTime: 1800,
      tags: ['memory-leak', 'websocket', 'performance'],
      resolved: false
    },
    {
      id: 'issue-003',
      title: 'CSS loading race condition on page refresh',
      severity: 'medium',
      environment: 'development',
      timestamp: '2025-08-08T21:30:45Z',
      description: 'Styles not loading correctly on hard refresh causing layout issues',
      stackTrace: 'Error: Failed to load resource: ade-main.css\n  at HTMLLinkElement.onError (line 7)',
      aiAnalysis: {
        rootCause: 'CSS file not embedded correctly in Rust binary static assets',
        suggestions: [
          'Verify CSS file path in include_str! macro',
          'Add fallback CSS loading mechanism',
          'Implement CSS-in-JS for critical styles',
          'Check build process for asset embedding'
        ],
        confidence: 0.78
      },
      responseTime: 950,
      tags: ['css', 'assets', 'loading'],
      resolved: true
    }
  ];

  $: filteredIssues = issues
    .filter(issue => filterEnvironment === 'all' || issue.environment === filterEnvironment)
    .filter(issue => filterSeverity === 'all' || issue.severity === filterSeverity)
    .sort((a, b) => {
      switch (sortBy) {
        case 'severity':
          const severityOrder = { critical: 0, high: 1, medium: 2, low: 3 };
          return severityOrder[a.severity] - severityOrder[b.severity];
        case 'timestamp':
          return new Date(b.timestamp) - new Date(a.timestamp);
        case 'responseTime':
          return b.responseTime - a.responseTime;
        default:
          return 0;
      }
    });

  $: issueStats = {
    total: issues.length,
    critical: issues.filter(i => i.severity === 'critical').length,
    high: issues.filter(i => i.severity === 'high').length,
    medium: issues.filter(i => i.severity === 'medium').length,
    low: issues.filter(i => i.severity === 'low').length,
    resolved: issues.filter(i => i.resolved).length,
    avgResponseTime: Math.round(issues.reduce((sum, i) => sum + i.responseTime, 0) / issues.length || 0)
  };

  function selectIssue(issue) {
    selectedIssue = issue;
  }

  function getSeverityColor(severity) {
    switch (severity) {
      case 'critical': return 'var(--color-error)';
      case 'high': return 'var(--color-error)';
      case 'medium': return 'var(--color-warning)';
      case 'low': return 'var(--color-success)';
      default: return 'var(--color-text-secondary)';
    }
  }

  function getEnvironmentColor(env) {
    switch (env) {
      case 'production': return 'var(--color-error)';
      case 'test': return 'var(--color-warning)';
      case 'development': return 'var(--color-success)';
      default: return 'var(--color-text-secondary)';
    }
  }

  function formatTimestamp(timestamp) {
    return new Date(timestamp).toLocaleString();
  }

  function markResolved(issueId) {
    const issue = issues.find(i => i.id === issueId);
    if (issue) {
      issue.resolved = true;
      issues = [...issues]; // Trigger reactivity
    }
  }

  function createTask(issueId) {
    const issue = issues.find(i => i.id === issueId);
    if (issue) {
      console.log('Creating task for issue:', issue.title);
      // Integration with task management would go here
    }
  }

  onMount(() => {
    issues = $issuesStore.length > 0 ? $issuesStore : sampleIssues;
    if (issues.length > 0 && !selectedIssue) {
      selectIssue(issues[0]);
    }
  });
</script>

<div class="issues-diagnostics-container">
  <div class="issues-header card bg-surface">
    <h1 class="text-primary">Issues & Diagnostics</h1>
    
    <div class="issue-stats">
      <div class="stat-card card bg-surface-2">
        <div class="stat-number text-error">{issueStats.critical}</div>
        <div class="stat-label text-secondary">Critical</div>
      </div>
      <div class="stat-card card bg-surface-2">
        <div class="stat-number text-error">{issueStats.high}</div>
        <div class="stat-label text-secondary">High</div>
      </div>
      <div class="stat-card card bg-surface-2">
        <div class="stat-number text-warning">{issueStats.medium}</div>
        <div class="stat-label text-secondary">Medium</div>
      </div>
      <div class="stat-card card bg-surface-2">
        <div class="stat-number text-success">{issueStats.resolved}</div>
        <div class="stat-label text-secondary">Resolved</div>
      </div>
      <div class="stat-card card bg-surface-2">
        <div class="stat-number text-info">{issueStats.avgResponseTime}ms</div>
        <div class="stat-label text-secondary">Avg Response</div>
      </div>
    </div>
  </div>

  <div class="issues-interface">
    <!-- Left Panel - Issues List -->
    <div class="issues-panel card bg-surface">
      <div class="panel-header">
        <h2>Active Issues</h2>
        
        <div class="filters">
          <select bind:value={filterEnvironment} class="filter-select bg-surface border rounded-md text-primary">
            <option value="all">All Environments</option>
            <option value="production">Production</option>
            <option value="test">Test</option>
            <option value="development">Development</option>
          </select>
          
          <select bind:value={filterSeverity} class="filter-select bg-surface border rounded-md text-primary">
            <option value="all">All Severities</option>
            <option value="critical">Critical</option>
            <option value="high">High</option>
            <option value="medium">Medium</option>
            <option value="low">Low</option>
          </select>
          
          <select bind:value={sortBy} class="filter-select bg-surface border rounded-md text-primary">
            <option value="timestamp">Sort by Time</option>
            <option value="severity">Sort by Severity</option>
            <option value="responseTime">Sort by Performance</option>
          </select>
        </div>
      </div>
      
      <div class="issues-list">
        {#each filteredIssues as issue}
          <div 
            class="issue-card card bg-surface-2 border rounded-md" 
            class:selected={selectedIssue?.id === issue.id}
            class:resolved={issue.resolved}
            on:click={() => selectIssue(issue)}
          >
            <div class="issue-header">
              <div class="severity-badge text-primary bg-surface-3 rounded-lg">
                {issue.severity.toUpperCase()}
              </div>
              <div class="environment-badge text-secondary">
                {issue.environment}
              </div>
            </div>
            
            <div class="issue-title text-primary">{issue.title}</div>
            <div class="issue-meta">
              <span class="issue-time text-secondary">{formatTimestamp(issue.timestamp)}</span>
              <span class="issue-response text-tertiary">{issue.responseTime}ms</span>
            </div>
            
            <div class="issue-tags">
              {#each issue.tags as tag}
                <span class="tag bg-surface-3 text-secondary rounded-md">{tag}</span>
              {/each}
            </div>
          </div>
        {/each}
      </div>
    </div>

    <!-- Right Panel - Issue Details -->
    <div class="details-panel card bg-surface">
      {#if selectedIssue}
        <div class="detail-header">
          <div class="detail-title text-primary">{selectedIssue.title}</div>
          <div class="detail-actions">
            <button 
              class="btn-secondary" 
              class:disabled={selectedIssue.resolved}
              on:click={() => markResolved(selectedIssue.id)}
            >
              {selectedIssue.resolved ? 'Resolved' : 'Mark Resolved'}
            </button>
            <button class="btn-primary" on:click={() => createTask(selectedIssue.id)}>
              Create Task
            </button>
          </div>
        </div>

        <div class="detail-content">
          <div class="detail-section">
            <h3 class="text-primary">Description</h3>
            <p class="description text-secondary">{selectedIssue.description}</p>
          </div>

          <div class="detail-section">
            <h3 class="text-primary">Stack Trace</h3>
            <pre class="stack-trace bg-surface-2 border rounded-md text-error">{selectedIssue.stackTrace}</pre>
          </div>

          <div class="detail-section">
            <h3 class="text-primary">AI Root Cause Analysis</h3>
            <div class="ai-analysis card bg-surface-2 border rounded-md">
              <div class="analysis-confidence text-info">
                Confidence: {Math.round(selectedIssue.aiAnalysis.confidence * 100)}%
              </div>
              <div class="root-cause">
                <strong class="text-primary">Root Cause:</strong> <span class="text-secondary">{selectedIssue.aiAnalysis.rootCause}</span>
              </div>
              <div class="suggestions">
                <strong class="text-primary">Suggested Fixes:</strong>
                <ul>
                  {#each selectedIssue.aiAnalysis.suggestions as suggestion}
                    <li class="text-secondary">{suggestion}</li>
                  {/each}
                </ul>
              </div>
            </div>
          </div>

          <div class="detail-section">
            <h3 class="text-primary">Performance Impact</h3>
            <div class="performance-metrics">
              <div class="metric">
                <span class="metric-label text-secondary">Response Time:</span>
                <span class="metric-value text-primary">{selectedIssue.responseTime}ms</span>
              </div>
              <div class="metric">
                <span class="metric-label text-secondary">Environment:</span>
                <span class="metric-value text-primary">
                  {selectedIssue.environment}
                </span>
              </div>
              <div class="metric">
                <span class="metric-label text-secondary">Severity:</span>
                <span class="metric-value text-primary">
                  {selectedIssue.severity}
                </span>
              </div>
            </div>
          </div>
        </div>
      {:else}
        <div class="no-selection">
          <p class="text-secondary">Select an issue to view details and AI analysis</p>
        </div>
      {/if}
    </div>
  </div>
</div>

