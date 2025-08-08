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
      color: '#666666',
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

<style>
  .feature-metrics-card {
    background: linear-gradient(135deg, #1a1a1a 0%, #2a2a2a 100%);
    border: 1px solid #333;
    border-radius: 12px;
    padding: 24px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  .feature-metrics-card h2 {
    color: #9b59d0;
    margin: 0 0 20px 0;
    font-size: 20px;
    font-weight: 600;
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 16px;
    margin-bottom: 24px;
  }

  .metric-item {
    background: rgba(255, 255, 255, 0.05);
    padding: 16px;
    border-radius: 8px;
    text-align: center;
  }

  .metric-value {
    font-size: 24px;
    font-weight: 700;
    color: #ffffff;
    margin-bottom: 4px;
  }

  .metric-value.implemented {
    color: #f59e0b;
  }

  .metric-value.tested {
    color: #4ade80;
  }

  .metric-value.percentage {
    color: #9b59d0;
  }

  .metric-label {
    font-size: 12px;
    color: #888;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .progress-section {
    margin-bottom: 24px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .progress-item {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .progress-header {
    display: flex;
    justify-content: space-between;
    font-size: 13px;
  }

  .progress-label {
    color: #ccc;
  }

  .progress-value {
    color: #9b59d0;
    font-weight: 600;
  }

  .progress-bar {
    height: 6px;
    background: #333;
    border-radius: 3px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    transition: width 0.6s ease;
  }

  .progress-fill.implementation {
    background: linear-gradient(90deg, #f59e0b, #fbbf24);
  }

  .progress-fill.testing {
    background: linear-gradient(90deg, #4ade80, #6ee7b7);
  }

  .chart-section {
    border-top: 1px solid #333;
    padding-top: 20px;
  }

  .chart-section h3 {
    color: #ccc;
    font-size: 14px;
    font-weight: 600;
    margin: 0 0 16px 0;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .svg-chart-container {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 20px;
  }

  .donut-segment {
    opacity: 0;
    animation: drawSegment 1s ease-in-out forwards;
    animation-delay: var(--delay, 0s);
  }

  .donut-center-text {
    fill: #ffffff;
    font-size: 20px;
    font-weight: 700;
  }

  .donut-center-label {
    fill: #888;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .chart-legend {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
  }

  .legend-color {
    width: 12px;
    height: 12px;
    border-radius: 50%;
  }

  .legend-label {
    color: #ccc;
    flex: 1;
  }

  .legend-value {
    color: #9b59d0;
    font-weight: 600;
    font-family: monospace;
  }

  @keyframes drawSegment {
    from {
      opacity: 0;
      stroke-dasharray: 0 1000;
    }
    to {
      opacity: 1;
      stroke-dasharray: 1000 0;
    }
  }

  @media (max-width: 640px) {
    .svg-chart-container {
      flex-direction: column;
      align-items: center;
    }
    
    .metrics-grid {
      grid-template-columns: 1fr;
    }
  }
</style>