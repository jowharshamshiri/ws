<script>
  import { featureMetrics, featuresStore } from '../stores.js';
  
  $: metrics = $featureMetrics;

  function polarToCartesian(centerX, centerY, radius, angleInDegrees) {
    const angleInRadians = (angleInDegrees - 90) * Math.PI / 180.0;
    return {
      x: centerX + (radius * Math.cos(angleInRadians)),
      y: centerY + (radius * Math.sin(angleInRadians))
    };
  }

  function describeArc(x, y, radius, startAngle, endAngle) {
    const start = polarToCartesian(x, y, radius, endAngle);
    const end = polarToCartesian(x, y, radius, startAngle);
    const largeArcFlag = endAngle - startAngle <= 180 ? "0" : "1";
    return [
      "M", start.x, start.y, 
      "A", radius, radius, 0, largeArcFlag, 0, end.x, end.y
    ].join(" ");
  }

  $: donutData = [
    { 
      label: 'Tested', 
      value: metrics.tested, 
      percentage: metrics.testCoveragePercentage,
      color: '#4ade80',
      startAngle: 0
    },
    { 
      label: 'Implemented', 
      value: metrics.implemented - metrics.tested, 
      percentage: Math.round(((metrics.implemented - metrics.tested) / metrics.total) * 100),
      color: '#f59e0b',
      startAngle: 0
    },
    { 
      label: 'Pending', 
      value: metrics.total - metrics.implemented, 
      percentage: Math.round(((metrics.total - metrics.implemented) / metrics.total) * 100),
      color: 'var(--color-text-secondary)',
      startAngle: 0
    }
  ];

  $: {
    // Calculate start angles for each segment
    let currentAngle = 0;
    donutData.forEach((segment, i) => {
      segment.startAngle = currentAngle;
      segment.endAngle = currentAngle + (segment.value / metrics.total) * 360;
      currentAngle = segment.endAngle;
    });
  }
</script>

<div class="feature-metrics-card">
  <h2>Implementation Progress</h2>
  
  <div class="metrics-grid">
    <div class="metric-item">
      <div class="metric-value">{metrics.total}</div>
      <div class="metric-label">Total Features</div>
    </div>
    
    <div class="metric-item">
      <div class="metric-value implemented">{metrics.implemented}</div>
      <div class="metric-label">Implemented</div>
    </div>
    
    <div class="metric-item">
      <div class="metric-value tested">{metrics.tested}</div>
      <div class="metric-label">Tested</div>
    </div>
    
    <div class="metric-item">
      <div class="metric-value percentage">{metrics.implementationPercentage}%</div>
      <div class="metric-label">Complete</div>
    </div>
  </div>

  <div class="progress-section">
    <div class="progress-item">
      <div class="progress-header">
        <span class="progress-label">Implementation</span>
        <span class="progress-value">{metrics.implementationPercentage}%</span>
      </div>
      <div class="progress-bar">
        <div class="progress-fill implementation" style="width: {metrics.implementationPercentage}%"></div>
      </div>
    </div>
    
    <div class="progress-item">
      <div class="progress-header">
        <span class="progress-label">Test Coverage</span>
        <span class="progress-value">{metrics.testCoveragePercentage}%</span>
      </div>
      <div class="progress-bar">
        <div class="progress-fill testing" style="width: {metrics.testCoveragePercentage}%"></div>
      </div>
    </div>
  </div>

  <div class="chart-section">
    <h3>Feature Distribution</h3>
    <div class="svg-chart-container">
      <svg width="200" height="200" viewBox="0 0 200 200">
        <g transform="translate(100, 100)">
          {#each donutData as segment, i}
            {#if segment.value > 0}
              <path 
                d={describeArc(0, 0, 70, segment.startAngle, segment.endAngle)}
                fill="none"
                stroke={segment.color}
                stroke-width="20"
                stroke-linecap="round"
                class="donut-segment"
                style="--delay: {i * 0.2}s"
              />
            {/if}
          {/each}
          
          <!-- Center text -->
          <text x="0" y="-5" text-anchor="middle" class="donut-center-text">
            {metrics.total}
          </text>
          <text x="0" y="10" text-anchor="middle" class="donut-center-label">
            Features
          </text>
        </g>
      </svg>
      
      <div class="chart-legend">
        {#each donutData as segment}
          {#if segment.value > 0}
            <div class="legend-item">
              <div class="legend-color" style="background-color: {segment.color}"></div>
              <span class="legend-label">{segment.label}</span>
              <span class="legend-value">{segment.value}</span>
            </div>
          {/if}
        {/each}
      </div>
    </div>
  </div>
</div>

