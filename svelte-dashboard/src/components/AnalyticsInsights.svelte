<script>
  import { onMount } from 'svelte';

  let selectedTimeframe = '30d';
  let selectedMetric = 'velocity';
  let chartMode = 'overview';
  
  const timeframes = [
    { id: '7d', name: '7 Days', period: 'Last Week' },
    { id: '30d', name: '30 Days', period: 'Last Month' },
    { id: '90d', name: '90 Days', period: 'Last Quarter' },
    { id: '1y', name: '1 Year', period: 'Last Year' }
  ];

  const metrics = [
    { id: 'velocity', name: 'Development Velocity', icon: '🚀' },
    { id: 'quality', name: 'Code Quality', icon: '⭐' },
    { id: 'ai_effectiveness', name: 'AI Effectiveness', icon: '🤖' },
    { id: 'completion_time', name: 'Completion Time', icon: '⏱️' },
    { id: 'resource_usage', name: 'Resource Usage', icon: '📊' }
  ];

  // Mock analytics data
  const analyticsData = {
    velocity: {
      current: 12.8,
      previous: 9.4,
      trend: 'up',
      unit: 'features/week',
      chart: [6, 8, 10, 12, 14, 16, 18, 20, 18, 16, 14, 12],
      breakdown: [
        { category: 'Features', count: 89, change: 15 },
        { category: 'Bug Fixes', count: 34, change: -8 },
        { category: 'Enhancements', count: 67, change: 22 }
      ]
    },
    quality: {
      current: 94,
      previous: 91,
      trend: 'up',
      unit: '/100 score',
      chart: [88, 89, 91, 93, 94, 96, 95, 94, 96, 97, 95, 94],
      breakdown: [
        { category: 'Test Coverage', score: 89, status: 'good' },
        { category: 'Code Standards', score: 96, status: 'excellent' },
        { category: 'Security Score', score: 98, status: 'excellent' },
        { category: 'Performance', score: 93, status: 'good' }
      ]
    },
    ai_effectiveness: {
      current: 92,
      previous: 87,
      trend: 'up',
      unit: '% success',
      chart: [78, 82, 85, 88, 90, 91, 89, 92, 94, 93, 91, 92],
      breakdown: [
        { metric: 'First Attempt Success', value: 87, change: 12 },
        { metric: 'Error Recovery Rate', value: 94, change: 8 },
        { metric: 'Context Efficiency', value: 76, change: 15 },
        { metric: 'Manual Intervention', value: 8, change: -23 }
      ]
    },
    completion_time: {
      current: 3.2,
      previous: 4.1,
      trend: 'down',
      unit: 'hours avg',
      chart: [5.2, 4.8, 4.5, 4.2, 3.9, 3.6, 3.4, 3.2, 3.1, 3.3, 3.4, 3.2],
      breakdown: [
        { type: 'Simple Features', time: 1.8, target: 2.0 },
        { type: 'Complex Features', time: 6.4, target: 8.0 },
        { type: 'Bug Fixes', time: 0.8, target: 1.0 },
        { type: 'Refactoring', time: 4.2, target: 5.0 }
      ]
    },
    resource_usage: {
      current: 47,
      previous: 52,
      trend: 'down',
      unit: '% context',
      chart: [62, 58, 55, 53, 51, 49, 48, 47, 46, 48, 49, 47],
      breakdown: [
        { resource: 'Context Window', usage: 47, limit: 100 },
        { resource: 'Session Duration', usage: 3.4, limit: 8.0 },
        { resource: 'Tool Calls', usage: 245, limit: 500 },
        { resource: 'Memory Usage', usage: 34, limit: 100 }
      ]
    }
  };

  const predictions = {
    features_completion: {
      date: '2025-08-22',
      confidence: 85,
      remaining: 47
    },
    quality_target: {
      score: 96,
      timeline: '2 weeks',
      confidence: 78
    },
    milestone_completion: {
      name: 'ADE Interface Complete',
      date: '2025-08-15',
      confidence: 92,
      progress: 59
    }
  };

  let currentData = analyticsData.velocity;

  onMount(() => {
    updateMetricData();
  });

  function selectTimeframe(timeframe) {
    selectedTimeframe = timeframe;
    updateMetricData();
  }

  function selectMetric(metric) {
    selectedMetric = metric;
    currentData = analyticsData[metric];
    updateMetricData();
  }

  function updateMetricData() {
    currentData = analyticsData[selectedMetric];
  }

  function toggleChartMode() {
    chartMode = chartMode === 'overview' ? 'detailed' : 'overview';
  }

  function getTrendIcon(trend) {
    return trend === 'up' ? '↗️' : trend === 'down' ? '↘️' : '→';
  }

  function getTrendColor(trend) {
    return trend === 'up' ? 'var(--success-color, #10b981)' : trend === 'down' ? 'var(--error-color, #ef4444)' : 'var(--text-secondary, #6b7280)';
  }

  function getStatusColor(status) {
    const colors = {
      excellent: 'var(--success-color, #10b981)',
      good: 'var(--warning-color, #f59e0b)',
      fair: 'var(--error-color, #ef4444)',
      poor: 'var(--error-color, #ef4444)'
    };
    return colors[status] || 'var(--text-secondary, #6b7280)';
  }
</script>

<div class="analytics-insights">
  <!-- Header -->
  <div class="analytics-header">
    <div class="header-left">
      <h2>Analytics & Insights</h2>
      <p class="subtitle">Development metrics, AI effectiveness, and performance predictions</p>
    </div>
    <div class="header-controls">
      <div class="timeframe-selector">
        {#each timeframes as timeframe}
          <button 
            class="timeframe-btn {selectedTimeframe === timeframe.id ? 'active' : ''}"
            on:click={() => selectTimeframe(timeframe.id)}
          >
            {timeframe.name}
          </button>
        {/each}
      </div>
    </div>
  </div>

  <!-- Metric Navigation -->
  <div class="metric-navigation">
    {#each metrics as metric}
      <button 
        class="metric-btn {selectedMetric === metric.id ? 'active' : ''}"
        on:click={() => selectMetric(metric.id)}
      >
        <span class="metric-icon">{metric.icon}</span>
        <span class="metric-name">{metric.name}</span>
      </button>
    {/each}
  </div>

  <!-- Main Content -->
  <div class="analytics-content">
    <!-- Primary Metrics Dashboard -->
    <div class="metrics-overview">
      <div class="primary-metric">
        <div class="metric-header">
          <h3>{metrics.find(m => m.id === selectedMetric)?.name}</h3>
          <button class="chart-toggle" on:click={toggleChartMode}>
            {chartMode === 'overview' ? 'Detailed' : 'Overview'}
          </button>
        </div>
        
        <div class="metric-value">
          <span class="current-value">{currentData.current}</span>
          <span class="metric-unit">{currentData.unit}</span>
          <div class="trend-indicator" style="color: {getTrendColor(currentData.trend)}">
            <span class="trend-icon">{getTrendIcon(currentData.trend)}</span>
            <span class="trend-change">
              {Math.abs(((currentData.current - currentData.previous) / currentData.previous) * 100).toFixed(1)}%
            </span>
          </div>
        </div>

        <div class="metric-chart">
          <div class="chart-container">
            <svg viewBox="0 0 400 100" class="trend-chart">
              {#each currentData.chart as value, i}
                <rect
                  x={i * 32}
                  y={100 - (value / Math.max(...currentData.chart)) * 80}
                  width="28"
                  height={(value / Math.max(...currentData.chart)) * 80}
                  fill="var(--accent-color, #3b82f6)"
                  opacity="0.8"
                />
              {/each}
            </svg>
          </div>
        </div>
      </div>

      <!-- Secondary Metrics Grid -->
      <div class="secondary-metrics">
        {#if selectedMetric === 'velocity'}
          <div class="metric-breakdown">
            <h4>Feature Breakdown</h4>
            {#each currentData.breakdown as item}
              <div class="breakdown-item">
                <span class="breakdown-label">{item.category}</span>
                <span class="breakdown-count">{item.count}</span>
                <span class="breakdown-change" style="color: {item.change > 0 ? 'var(--success-color, #10b981)' : 'var(--error-color, #ef4444)'}">
                  {item.change > 0 ? '+' : ''}{item.change}%
                </span>
              </div>
            {/each}
          </div>
        {:else if selectedMetric === 'quality'}
          <div class="metric-breakdown">
            <h4>Quality Metrics</h4>
            {#each currentData.breakdown as item}
              <div class="breakdown-item">
                <span class="breakdown-label">{item.category}</span>
                <span class="breakdown-score">{item.score}/100</span>
                <div class="status-badge" style="background: {getStatusColor(item.status)}">
                  {item.status}
                </div>
              </div>
            {/each}
          </div>
        {:else if selectedMetric === 'ai_effectiveness'}
          <div class="metric-breakdown">
            <h4>AI Performance</h4>
            {#each currentData.breakdown as item}
              <div class="breakdown-item">
                <span class="breakdown-label">{item.metric}</span>
                <span class="breakdown-value">{item.value}%</span>
                <span class="breakdown-change" style="color: {item.change > 0 ? 'var(--success-color, #10b981)' : 'var(--error-color, #ef4444)'}">
                  {item.change > 0 ? '+' : ''}{item.change}%
                </span>
              </div>
            {/each}
          </div>
        {:else if selectedMetric === 'completion_time'}
          <div class="metric-breakdown">
            <h4>Time by Feature Type</h4>
            {#each currentData.breakdown as item}
              <div class="breakdown-item">
                <span class="breakdown-label">{item.type}</span>
                <span class="breakdown-time">{item.time}h</span>
                <div class="target-comparison">
                  Target: {item.target}h
                </div>
              </div>
            {/each}
          </div>
        {:else if selectedMetric === 'resource_usage'}
          <div class="metric-breakdown">
            <h4>Resource Utilization</h4>
            {#each currentData.breakdown as item}
              <div class="breakdown-item">
                <span class="breakdown-label">{item.resource}</span>
                <div class="usage-bar">
                  <div 
                    class="usage-fill" 
                    style="width: {(item.usage / item.limit) * 100}%"
                  ></div>
                </div>
                <span class="usage-text">{item.usage}/{item.limit}</span>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>

    <!-- Predictions Panel -->
    <div class="predictions-panel">
      <div class="panel-header">
        <h3>Completion Predictions</h3>
        <div class="prediction-note">Based on current velocity and trends</div>
      </div>
      
      <div class="predictions-grid">
        <div class="prediction-card">
          <h4>Features Complete</h4>
          <div class="prediction-date">{predictions.features_completion.date}</div>
          <div class="prediction-details">
            <span class="remaining">{predictions.features_completion.remaining} remaining</span>
            <span class="confidence">{predictions.features_completion.confidence}% confidence</span>
          </div>
        </div>

        <div class="prediction-card">
          <h4>Quality Target</h4>
          <div class="prediction-score">{predictions.quality_target.score}/100</div>
          <div class="prediction-details">
            <span class="timeline">{predictions.quality_target.timeline}</span>
            <span class="confidence">{predictions.quality_target.confidence}% confidence</span>
          </div>
        </div>

        <div class="prediction-card">
          <h4>Next Milestone</h4>
          <div class="prediction-milestone">{predictions.milestone_completion.name}</div>
          <div class="prediction-details">
            <span class="milestone-date">{predictions.milestone_completion.date}</span>
            <span class="confidence">{predictions.milestone_completion.confidence}% confidence</span>
          </div>
          <div class="milestone-progress">
            <div class="progress-bar">
              <div 
                class="progress-fill" 
                style="width: {predictions.milestone_completion.progress}%"
              ></div>
            </div>
            <span class="progress-text">{predictions.milestone_completion.progress}%</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Custom Reports Section -->
    <div class="custom-reports">
      <div class="reports-header">
        <h3>Custom Report Builder</h3>
        <button class="btn-secondary">Generate Report</button>
      </div>
      
      <div class="report-builder">
        <div class="builder-section">
          <label>Time Range</label>
          <select class="report-select">
            <option>Last 7 days</option>
            <option>Last 30 days</option>
            <option>Last quarter</option>
            <option>Custom range</option>
          </select>
        </div>
        
        <div class="builder-section">
          <label>Metrics to Include</label>
          <div class="metric-checkboxes">
            <label class="checkbox-item">
              <input type="checkbox" checked />
              Development Velocity
            </label>
            <label class="checkbox-item">
              <input type="checkbox" checked />
              AI Effectiveness
            </label>
            <label class="checkbox-item">
              <input type="checkbox" />
              Code Quality
            </label>
            <label class="checkbox-item">
              <input type="checkbox" />
              Resource Usage
            </label>
          </div>
        </div>
        
        <div class="builder-section">
          <label>Export Format</label>
          <div class="format-options">
            <button class="format-btn">PDF</button>
            <button class="format-btn">CSV</button>
            <button class="format-btn">JSON</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .analytics-insights {
    color: var(--text-primary, CanvasText);
    min-height: 100vh;
    padding: 1.5rem;
  }

  .analytics-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 2rem;
  }

  .header-left h2 {
    color: var(--text-primary, CanvasText);
    font-size: 1.875rem;
    font-weight: 700;
    margin: 0 0 0.5rem 0;
  }

  .subtitle {
    color: var(--text-secondary, #6b7280);
    font-size: 0.875rem;
    margin: 0;
  }

  .timeframe-selector {
    display: flex;
    gap: 0.25rem;
    background: var(--bg-secondary, Canvas);
    border: 1px solid var(--border-color, #e5e7eb);
    border-radius: 0.5rem;
    padding: 0.25rem;
  }

  .timeframe-btn {
    padding: 0.5rem 1rem;
    background: transparent;
    border: none;
    color: var(--text-primary, CanvasText);
    border-radius: 0.375rem;
    cursor: pointer;
    transition: all 0.2s;
    font-size: 0.875rem;
  }

  .timeframe-btn:hover {
    background: var(--hover-bg, #f3f4f6);
  }

  .timeframe-btn.active {
    background: var(--accent-color, #3b82f6);
    color: white;
  }

  .metric-navigation {
    display: flex;
    gap: 1rem;
    margin-bottom: 2rem;
    overflow-x: auto;
    padding-bottom: 0.5rem;
  }

  .metric-btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1.5rem;
    background: var(--bg-secondary, Canvas);
    border: 1px solid var(--border-color, #e5e7eb);
    border-radius: 0.5rem;
    color: var(--text-primary, CanvasText);
    cursor: pointer;
    transition: all 0.2s;
    white-space: nowrap;
  }

  .metric-btn:hover {
    background: var(--hover-bg, #f3f4f6);
  }

  .metric-btn.active {
    background: var(--accent-color, #3b82f6);
    border-color: var(--accent-color, #3b82f6);
    color: white;
  }

  .metric-icon {
    font-size: 1.25rem;
  }

  .metric-name {
    font-weight: 500;
  }

  .analytics-content {
    display: grid;
    gap: 2rem;
  }

  .metrics-overview {
    display: grid;
    grid-template-columns: 2fr 1fr;
    gap: 2rem;
  }

  .primary-metric {
    background: var(--bg-secondary, Canvas);
    border: 1px solid var(--border-color, #e5e7eb);
    border-radius: 0.75rem;
    padding: 2rem;
  }

  .metric-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
  }

  .metric-header h3 {
    color: var(--text-primary, CanvasText);
    font-size: 1.25rem;
    font-weight: 600;
    margin: 0;
  }

  .chart-toggle {
    background: var(--bg-tertiary, #f9fafb);
    border: 1px solid var(--border-color, #e5e7eb);
    color: var(--text-primary, CanvasText);
    padding: 0.5rem 1rem;
    border-radius: 0.375rem;
    cursor: pointer;
    font-size: 0.875rem;
  }

  .chart-toggle:hover {
    background: var(--hover-bg, #f3f4f6);
  }

  .metric-value {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    margin-bottom: 2rem;
  }

  .current-value {
    font-size: 3rem;
    font-weight: 700;
    color: var(--text-primary, CanvasText);
  }

  .metric-unit {
    font-size: 1.125rem;
    color: var(--text-secondary, #6b7280);
  }

  .trend-indicator {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    margin-left: auto;
  }

  .trend-icon {
    font-size: 1.25rem;
  }

  .trend-change {
    font-weight: 600;
  }

  .metric-chart {
    height: 100px;
  }

  .chart-container {
    height: 100%;
  }

  .trend-chart {
    width: 100%;
    height: 100%;
  }

  .secondary-metrics {
    background: var(--bg-secondary, Canvas);
    border: 1px solid var(--border-color, #e5e7eb);
    border-radius: 0.75rem;
    padding: 1.5rem;
  }

  .metric-breakdown h4 {
    color: var(--text-primary, CanvasText);
    font-size: 1rem;
    font-weight: 600;
    margin: 0 0 1rem 0;
  }

  .breakdown-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 0;
    border-bottom: 1px solid var(--border-color, #e5e7eb);
  }

  .breakdown-item:last-child {
    border-bottom: none;
  }

  .breakdown-label {
    color: var(--text-secondary, #6b7280);
    font-size: 0.875rem;
  }

  .breakdown-count, .breakdown-score, .breakdown-value, .breakdown-time {
    color: var(--text-primary, CanvasText);
    font-weight: 600;
  }

  .breakdown-change {
    font-size: 0.75rem;
    font-weight: 600;
  }

  .status-badge {
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
    color: white;
    font-size: 0.75rem;
    font-weight: 500;
    text-transform: capitalize;
  }

  .target-comparison {
    color: var(--text-tertiary, #9ca3af);
    font-size: 0.75rem;
  }

  .usage-bar {
    width: 60px;
    height: 8px;
    background: var(--bg-tertiary, #f9fafb);
    border-radius: 4px;
    overflow: hidden;
  }

  .usage-fill {
    height: 100%;
    background: var(--accent-color, #3b82f6);
    transition: width 0.3s;
  }

  .usage-text {
    color: var(--text-secondary, #6b7280);
    font-size: 0.75rem;
    font-family: monospace;
  }

  .predictions-panel {
    background: var(--bg-secondary, Canvas);
    border: 1px solid var(--border-color, #e5e7eb);
    border-radius: 0.75rem;
    padding: 2rem;
  }

  .panel-header {
    margin-bottom: 1.5rem;
  }

  .panel-header h3 {
    color: var(--text-primary, CanvasText);
    font-size: 1.25rem;
    font-weight: 600;
    margin: 0 0 0.5rem 0;
  }

  .prediction-note {
    color: var(--text-secondary, #6b7280);
    font-size: 0.875rem;
  }

  .predictions-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1.5rem;
  }

  .prediction-card {
    background: var(--bg-tertiary, #f9fafb);
    border: 1px solid var(--border-color, #e5e7eb);
    border-radius: 0.5rem;
    padding: 1.5rem;
  }

  .prediction-card h4 {
    color: var(--text-primary, CanvasText);
    font-size: 1rem;
    font-weight: 600;
    margin: 0 0 1rem 0;
  }

  .prediction-date, .prediction-score, .prediction-milestone {
    color: var(--accent-color, #3b82f6);
    font-size: 1.25rem;
    font-weight: 700;
    margin-bottom: 0.75rem;
  }

  .prediction-details {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font-size: 0.875rem;
  }

  .remaining, .timeline, .milestone-date {
    color: var(--text-primary, CanvasText);
  }

  .confidence {
    color: var(--text-secondary, #6b7280);
  }

  .milestone-progress {
    margin-top: 1rem;
  }

  .progress-bar {
    width: 100%;
    height: 8px;
    background: var(--bg-tertiary, #f9fafb);
    border-radius: 4px;
    overflow: hidden;
    margin-bottom: 0.5rem;
  }

  .progress-fill {
    height: 100%;
    background: var(--success-color, #10b981);
    transition: width 0.3s;
  }

  .progress-text {
    color: var(--text-secondary, #6b7280);
    font-size: 0.75rem;
    font-weight: 600;
  }

  .custom-reports {
    background: var(--bg-secondary, Canvas);
    border: 1px solid var(--border-color, #e5e7eb);
    border-radius: 0.75rem;
    padding: 2rem;
  }

  .reports-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
  }

  .reports-header h3 {
    color: var(--text-primary, CanvasText);
    font-size: 1.25rem;
    font-weight: 600;
    margin: 0;
  }

  .btn-secondary {
    background: var(--bg-tertiary, #f9fafb);
    border: 1px solid var(--border-color, #e5e7eb);
    color: var(--text-primary, CanvasText);
    padding: 0.75rem 1.5rem;
    border-radius: 0.5rem;
    cursor: pointer;
    font-weight: 500;
    transition: background 0.2s;
  }

  .btn-secondary:hover {
    background: var(--hover-bg, #f3f4f6);
  }

  .report-builder {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 2rem;
  }

  .builder-section label {
    display: block;
    color: var(--text-primary, CanvasText);
    font-weight: 500;
    margin-bottom: 0.75rem;
  }

  .report-select {
    width: 100%;
    padding: 0.75rem;
    background: var(--bg-primary, Canvas);
    border: 1px solid var(--border-color, #e5e7eb);
    border-radius: 0.5rem;
    color: var(--text-primary, CanvasText);
  }

  .metric-checkboxes {
    display: grid;
    gap: 0.5rem;
  }

  .checkbox-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: var(--text-primary, CanvasText);
    cursor: pointer;
  }

  .checkbox-item input[type="checkbox"] {
    margin: 0;
  }

  .format-options {
    display: flex;
    gap: 0.5rem;
  }

  .format-btn {
    padding: 0.5rem 1rem;
    background: var(--bg-tertiary, #f9fafb);
    border: 1px solid var(--border-color, #e5e7eb);
    color: var(--text-primary, CanvasText);
    border-radius: 0.375rem;
    cursor: pointer;
    transition: background 0.2s;
  }

  .format-btn:hover {
    background: var(--hover-bg, #f3f4f6);
  }
</style>