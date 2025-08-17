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
    { id: 'velocity', name: 'Development Velocity', icon: 'V' },
    { id: 'quality', name: 'Code Quality', icon: 'Q' },
    { id: 'ai_effectiveness', name: 'AI Effectiveness', icon: 'A' },
    { id: 'completion_time', name: 'Completion Time', icon: 'T' },
    { id: 'resource_usage', name: 'Resource Usage', icon: 'R' }
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
    return trend === 'up' ? 'var(--color-success)' : trend === 'down' ? 'var(--color-error)' : 'var(--color-text-secondary)';
  }

  function getStatusColor(status) {
    const colors = {
      excellent: 'var(--color-success)',
      good: 'var(--color-warning)',
      fair: 'var(--color-error)',
      poor: 'var(--color-error)'
    };
    return colors[status] || 'var(--color-text-secondary)';
  }
</script>

<div class="analytics-insights-container card bg-surface">
  <!-- Header -->
  <div class="analytics-header">
    <div class="header-left">
      <h2 class="text-primary">Analytics & Insights</h2>
      <p class="subtitle text-secondary">Development metrics, AI effectiveness, and performance predictions</p>
    </div>
    <div class="header-controls">
      <div class="timeframe-selector">
        {#each timeframes as timeframe}
          <button 
            class="btn-secondary timeframe-btn {selectedTimeframe === timeframe.id ? 'active' : ''}"
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
        class="btn-secondary metric-btn {selectedMetric === metric.id ? 'active' : ''}"
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
      <div class="primary-metric card bg-surface-2">
        <div class="metric-header">
          <h3 class="text-primary">{metrics.find(m => m.id === selectedMetric)?.name}</h3>
          <button class="btn-secondary chart-toggle" on:click={toggleChartMode}>
            {chartMode === 'overview' ? 'Detailed' : 'Overview'}
          </button>
        </div>
        
        <div class="metric-value">
          <span class="current-value text-primary">{currentData.current}</span>
          <span class="metric-unit text-secondary">{currentData.unit}</span>
          <div class="trend-indicator trend-{currentData.trend}">
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
                  fill="var(--color-primary)"
                  opacity="0.8"
                />
              {/each}
            </svg>
          </div>
        </div>
      </div>

      <!-- Secondary Metrics Grid -->
      <div class="secondary-metrics card bg-surface-2">
        {#if selectedMetric === 'velocity'}
          <div class="metric-breakdown">
            <h4 class="text-primary">Feature Breakdown</h4>
            {#each currentData.breakdown as item}
              <div class="breakdown-item">
                <span class="breakdown-label text-secondary">{item.category}</span>
                <span class="breakdown-count text-primary">{item.count}</span>
                <span class="breakdown-change change-{item.change > 0 ? 'positive' : 'negative'}">
                  {item.change > 0 ? '+' : ''}{item.change}%
                </span>
              </div>
            {/each}
          </div>
        {:else if selectedMetric === 'quality'}
          <div class="metric-breakdown">
            <h4 class="text-primary">Quality Metrics</h4>
            {#each currentData.breakdown as item}
              <div class="breakdown-item">
                <span class="breakdown-label text-secondary">{item.category}</span>
                <span class="breakdown-score text-primary">{item.score}/100</span>
                <div class="status-badge status-{item.status}">
                  {item.status}
                </div>
              </div>
            {/each}
          </div>
        {:else if selectedMetric === 'ai_effectiveness'}
          <div class="metric-breakdown">
            <h4 class="text-primary">AI Performance</h4>
            {#each currentData.breakdown as item}
              <div class="breakdown-item">
                <span class="breakdown-label text-secondary">{item.metric}</span>
                <span class="breakdown-value text-primary">{item.value}%</span>
                <span class="breakdown-change change-{item.change > 0 ? 'positive' : 'negative'}">
                  {item.change > 0 ? '+' : ''}{item.change}%
                </span>
              </div>
            {/each}
          </div>
        {:else if selectedMetric === 'completion_time'}
          <div class="metric-breakdown">
            <h4 class="text-primary">Time by Feature Type</h4>
            {#each currentData.breakdown as item}
              <div class="breakdown-item">
                <span class="breakdown-label text-secondary">{item.type}</span>
                <span class="breakdown-time text-primary">{item.time}h</span>
                <div class="target-comparison">
                  <span class="text-tertiary">Target: {item.target}h</span>
                </div>
              </div>
            {/each}
          </div>
        {:else if selectedMetric === 'resource_usage'}
          <div class="metric-breakdown">
            <h4 class="text-primary">Resource Utilization</h4>
            {#each currentData.breakdown as item}
              <div class="breakdown-item">
                <span class="breakdown-label text-secondary">{item.resource}</span>
                <div class="usage-bar">
                  <div 
                    class="usage-fill" 
                    style:width="{(item.usage / item.limit) * 100}%"
                  ></div>
                </div>
                <span class="usage-text text-secondary">{item.usage}/{item.limit}</span>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>

    <!-- Predictions Panel -->
    <div class="predictions-panel card bg-surface-2">
      <div class="panel-header">
        <h3 class="text-primary">Completion Predictions</h3>
        <div class="prediction-note text-secondary">Based on current velocity and trends</div>
      </div>
      
      <div class="predictions-grid">
        <div class="prediction-card">
          <h4 class="text-secondary">Features Complete</h4>
          <div class="prediction-date text-primary">{predictions.features_completion.date}</div>
          <div class="prediction-details">
            <span class="remaining text-secondary">{predictions.features_completion.remaining} remaining</span>
            <span class="confidence text-tertiary">{predictions.features_completion.confidence}% confidence</span>
          </div>
        </div>

        <div class="prediction-card">
          <h4 class="text-secondary">Quality Target</h4>
          <div class="prediction-score text-primary">{predictions.quality_target.score}/100</div>
          <div class="prediction-details">
            <span class="timeline text-secondary">{predictions.quality_target.timeline}</span>
            <span class="confidence text-tertiary">{predictions.quality_target.confidence}% confidence</span>
          </div>
        </div>

        <div class="prediction-card">
          <h4 class="text-secondary">Next Milestone</h4>
          <div class="prediction-milestone text-primary">{predictions.milestone_completion.name}</div>
          <div class="prediction-details">
            <span class="milestone-date text-secondary">{predictions.milestone_completion.date}</span>
            <span class="confidence text-tertiary">{predictions.milestone_completion.confidence}% confidence</span>
          </div>
          <div class="milestone-progress">
            <div class="progress-bar">
              <div 
                class="progress-fill" 
                style:width="{predictions.milestone_completion.progress}%"
              ></div>
            </div>
            <span class="progress-text text-secondary">{predictions.milestone_completion.progress}%</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Custom Reports Section -->
    <div class="custom-reports card bg-surface-2">
      <div class="reports-header">
        <h3 class="text-primary">Custom Report Builder</h3>
        <button class="btn-primary">Generate Report</button>
      </div>
      
      <div class="report-builder">
        <div class="builder-section">
          <label class="text-secondary">Time Range</label>
          <select class="report-select bg-surface border rounded-md">
            <option>Last 7 days</option>
            <option>Last 30 days</option>
            <option>Last quarter</option>
            <option>Custom range</option>
          </select>
        </div>
        
        <div class="builder-section">
          <label class="text-secondary">Metrics to Include</label>
          <div class="metric-checkboxes">
            <label class="checkbox-item text-secondary">
              <input type="checkbox" checked />
              Development Velocity
            </label>
            <label class="checkbox-item text-secondary">
              <input type="checkbox" checked />
              AI Effectiveness
            </label>
            <label class="checkbox-item text-secondary">
              <input type="checkbox" />
              Code Quality
            </label>
            <label class="checkbox-item text-secondary">
              <input type="checkbox" />
              Resource Usage
            </label>
          </div>
        </div>
        
        <div class="builder-section">
          <label class="text-secondary">Export Format</label>
          <div class="format-options">
            <button class="btn-secondary format-btn">PDF</button>
            <button class="btn-secondary format-btn">CSV</button>
            <button class="btn-secondary format-btn">JSON</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

