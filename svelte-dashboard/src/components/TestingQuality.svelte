<script>
  import { onMount } from 'svelte';
  
  let testSuites = [];
  let selectedSuite = null;
  let testResults = [];
  let coverageData = { modules: [] };
  let performanceMetrics = [];
  let testHistory = [];
  let visualTests = [];
  let runningTests = false;
  let testOutput = '';
  let showOutput = false;

  // Sample test data
  onMount(() => {
    testSuites = [
      {
        id: 'unit',
        name: 'Unit Tests',
        type: 'unit',
        total: 247,
        passed: 243,
        failed: 4,
        skipped: 0,
        duration: 1.2,
        coverage: 94.2,
        status: 'completed',
        lastRun: '2025-01-08T12:30:45Z'
      },
      {
        id: 'integration',
        name: 'Integration Tests',
        type: 'integration',
        total: 89,
        passed: 85,
        failed: 2,
        skipped: 2,
        duration: 45.7,
        coverage: 87.5,
        status: 'completed',
        lastRun: '2025-01-08T12:25:30Z'
      },
      {
        id: 'visual',
        name: 'Visual Regression',
        type: 'visual',
        total: 156,
        passed: 152,
        failed: 4,
        skipped: 0,
        duration: 78.3,
        coverage: 0,
        status: 'running',
        lastRun: '2025-01-08T12:35:12Z'
      },
      {
        id: 'performance',
        name: 'Performance Tests',
        type: 'performance',
        total: 24,
        passed: 22,
        failed: 1,
        skipped: 1,
        duration: 124.6,
        coverage: 0,
        status: 'completed',
        lastRun: '2025-01-08T12:20:15Z'
      }
    ];

    testResults = [
      {
        suite: 'unit',
        name: 'user_authentication_test',
        status: 'passed',
        duration: 0.045,
        file: 'tests/auth/user_test.rs'
      },
      {
        suite: 'unit',
        name: 'password_validation_test',
        status: 'failed',
        duration: 0.023,
        file: 'tests/auth/password_test.rs',
        error: 'Expected password strength validation to fail for weak passwords'
      },
      {
        suite: 'integration',
        name: 'api_endpoint_creation',
        status: 'passed',
        duration: 2.1,
        file: 'tests/integration/api_test.rs'
      },
      {
        suite: 'visual',
        name: 'dashboard_layout_chrome',
        status: 'failed',
        duration: 3.4,
        file: 'tests/visual/dashboard_test.js',
        error: 'Pixel difference of 2.3% detected in header navigation'
      }
    ];

    coverageData = {
      overall: 91.4,
      modules: [
        { name: 'Authentication', coverage: 96.8, lines: 1247, uncovered: 40 },
        { name: 'API Endpoints', coverage: 89.2, lines: 2156, uncovered: 233 },
        { name: 'Database Layer', coverage: 94.5, lines: 987, uncovered: 54 },
        { name: 'UI Components', coverage: 87.3, lines: 1876, uncovered: 238 },
        { name: 'Utilities', coverage: 92.1, lines: 654, uncovered: 52 }
      ]
    };

    performanceMetrics = [
      {
        test: 'API Response Time',
        threshold: 200,
        actual: 156,
        status: 'passed',
        trend: '+3ms'
      },
      {
        test: 'Database Query Time',
        threshold: 50,
        actual: 78,
        status: 'failed',
        trend: '+12ms'
      },
      {
        test: 'Page Load Time',
        threshold: 2000,
        actual: 1234,
        status: 'passed',
        trend: '-45ms'
      },
      {
        test: 'Memory Usage',
        threshold: 100,
        actual: 87,
        status: 'passed',
        trend: '-2MB'
      }
    ];

    testHistory = [
      { date: '2025-01-08', passed: 350, failed: 7, total: 357 },
      { date: '2025-01-07', passed: 345, failed: 12, total: 357 },
      { date: '2025-01-06', passed: 342, failed: 15, total: 357 },
      { date: '2025-01-05', passed: 340, failed: 17, total: 357 },
      { date: '2025-01-04', passed: 338, failed: 19, total: 357 },
      { date: '2025-01-03', passed: 335, failed: 22, total: 357 },
      { date: '2025-01-02', passed: 330, failed: 27, total: 357 }
    ];

    visualTests = [
      {
        name: 'Login Page Layout',
        browser: 'Chrome',
        viewport: '1920x1080',
        status: 'passed',
        baseline: '/screenshots/login_baseline.png',
        current: '/screenshots/login_current.png',
        diff: 0.1
      },
      {
        name: 'Dashboard Header',
        browser: 'Firefox',
        viewport: '1920x1080',
        status: 'failed',
        baseline: '/screenshots/header_baseline.png',
        current: '/screenshots/header_current.png',
        diff: 2.3
      },
      {
        name: 'Mobile Navigation',
        browser: 'Chrome',
        viewport: '375x667',
        status: 'passed',
        baseline: '/screenshots/mobile_nav_baseline.png',
        current: '/screenshots/mobile_nav_current.png',
        diff: 0.05
      }
    ];
  });

  function runAllTests() {
    runningTests = true;
    testOutput = 'Starting test execution...\n';
    showOutput = true;
    
    setTimeout(() => {
      testOutput += 'Running unit tests...\n';
      testOutput += '243 tests passed\n';
      testOutput += '4 tests failed\n';
      testOutput += 'Running integration tests...\n';
      testOutput += '85 tests passed\n';
      testOutput += '2 tests failed\n';
      testOutput += '2 tests skipped\n';
      testOutput += '\nTest execution completed in 2m 34s\n';
      runningTests = false;
    }, 3000);
  }

  function runTestSuite(suite) {
    runningTests = true;
    testOutput = `Running ${suite.name}...\n`;
    showOutput = true;
    
    setTimeout(() => {
      testOutput += `${suite.passed} tests passed\n`;
      if (suite.failed > 0) {
        testOutput += `${suite.failed} tests failed\n`;
      }
      if (suite.skipped > 0) {
        testOutput += `${suite.skipped} tests skipped\n`;
      }
      testOutput += `\nCompleted in ${suite.duration}s\n`;
      runningTests = false;
    }, 2000);
  }

  function getStatusColor(status) {
    switch (status) {
      case 'passed': return 'var(--color-success)';
      case 'failed': return 'var(--color-error)';
      case 'skipped': return 'var(--color-warning)';
      case 'running': return 'var(--color-info)';
      default: return 'var(--color-text-secondary)';
    }
  }

  function getPassRate(suite) {
    return ((suite.passed / suite.total) * 100).toFixed(1);
  }

  function getCoverageColor(coverage) {
    if (coverage >= 80) return 'var(--color-success)';
    if (coverage >= 60) return 'var(--color-warning)';
    return 'var(--color-error)';
  }

  function getPerformanceStatus(metric) {
    return metric.actual <= metric.threshold ? 'passed' : 'failed';
  }

  function formatDuration(seconds) {
    if (seconds < 60) return `${seconds}s`;
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}m ${secs}s`;
  }

  function formatDate(dateString) {
    return new Date(dateString).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }
</script>

<div class="testing-quality-container card bg-surface">
  <!-- Header -->
  <div class="testing-header">
    <h1 class="text-primary">Testing & Quality</h1>
    <div class="header-controls">
      <button class="btn-primary run-all-btn" on:click={runAllTests} disabled={runningTests}>
        {#if runningTests}
          Running Tests...
        {:else}
          Run All Tests
        {/if}
      </button>
    </div>
  </div>

  <!-- Test Suite Overview -->
  <div class="suite-overview">
    <div class="overview-stats">
      {#each testSuites as suite}
        <div class="suite-card card bg-surface-2" class:running={suite.status === 'running'}>
          <div class="suite-header">
            <div class="suite-info">
              <h3 class="text-primary">{suite.name}</h3>
              <div class="suite-meta text-secondary">
                {suite.total} tests • {formatDuration(suite.duration)} • {formatDate(suite.lastRun)}
              </div>
            </div>
            <button class="btn-secondary run-suite-btn" on:click={() => runTestSuite(suite)} disabled={runningTests}>
              {#if suite.status === 'running'}
                Running
              {:else}
                Run
              {/if}
            </button>
          </div>
          
          <div class="suite-results">
            <div class="result-stats">
              <div class="stat passed">
                <div class="stat-value">{suite.passed}</div>
                <div class="stat-label text-secondary">Passed</div>
              </div>
              <div class="stat failed">
                <div class="stat-value">{suite.failed}</div>
                <div class="stat-label text-secondary">Failed</div>
              </div>
              {#if suite.skipped > 0}
                <div class="stat skipped">
                  <div class="stat-value">{suite.skipped}</div>
                  <div class="stat-label text-secondary">Skipped</div>
                </div>
              {/if}
            </div>
            
            <div class="pass-rate">
              <div class="rate-value status-{suite.failed === 0 ? 'passed' : 'failed'}">
                {getPassRate(suite)}%
              </div>
              <div class="rate-label text-secondary">Pass Rate</div>
            </div>
          </div>

          {#if suite.coverage > 0}
            <div class="coverage-info">
              <div class="coverage-label text-secondary">Code Coverage</div>
              <div class="coverage-bar">
                <div 
                  class="coverage-fill coverage-{suite.coverage >= 80 ? 'good' : suite.coverage >= 60 ? 'medium' : 'poor'}" 
                  style:width="{suite.coverage}%"
                ></div>
              </div>
              <div class="coverage-value coverage-{suite.coverage >= 80 ? 'good' : suite.coverage >= 60 ? 'medium' : 'poor'}">
                {suite.coverage}%
              </div>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </div>

  <!-- Main Content Grid -->
  <div class="testing-content">
    <!-- Test Results Panel -->
    <div class="test-results-panel">
      <div class="panel-header">
        <h2 class="text-primary">Recent Test Results</h2>
        <div class="result-filters">
          <select class="filter-select bg-surface border rounded-md">
            <option value="all">All Suites</option>
            <option value="unit">Unit Tests</option>
            <option value="integration">Integration</option>
            <option value="visual">Visual</option>
            <option value="performance">Performance</option>
          </select>
          <select class="filter-select bg-surface border rounded-md">
            <option value="all">All Results</option>
            <option value="failed">Failed Only</option>
            <option value="passed">Passed Only</option>
          </select>
        </div>
      </div>
      
      <div class="results-list">
        {#each testResults as result}
          <div class="result-item card bg-surface border" class:failed={result.status === 'failed'}>
            <div class="result-header">
              <div class="result-info">
                <div class="result-name text-primary">{result.name}</div>
                <div class="result-file text-secondary">{result.file}</div>
              </div>
              <div class="result-meta">
                <div class="result-status status-{result.status}">
                  {result.status}
                </div>
                <div class="result-duration">{result.duration}s</div>
              </div>
            </div>
            
            {#if result.error}
              <div class="result-error">
                <div class="error-message">{result.error}</div>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </div>

    <!-- Coverage Panel -->
    <div class="coverage-panel">
      <div class="panel-header">
        <h2 class="text-primary">Code Coverage</h2>
        <div class="coverage-overall">
          <span class="overall-value coverage-{coverageData.overall >= 80 ? 'good' : coverageData.overall >= 60 ? 'medium' : 'poor'}">
            {coverageData.overall}%
          </span>
          <span class="overall-label text-secondary">Overall</span>
        </div>
      </div>
      
      <div class="coverage-modules">
        {#each coverageData.modules as module}
          <div class="module-item">
            <div class="module-header">
              <div class="module-name text-primary">{module.name}</div>
              <div class="module-coverage coverage-{module.coverage >= 80 ? 'good' : module.coverage >= 60 ? 'medium' : 'poor'}">
                {module.coverage}%
              </div>
            </div>
            <div class="module-bar">
              <div 
                class="module-fill coverage-bar coverage-{module.coverage >= 80 ? 'good' : module.coverage >= 60 ? 'medium' : 'poor'}" 
                style:width="{module.coverage}%"
              ></div>
            </div>
            <div class="module-details">
              <span class="text-secondary">{module.lines - module.uncovered}/{module.lines} lines covered</span>
              <span class="uncovered-count text-tertiary">{module.uncovered} uncovered</span>
            </div>
          </div>
        {/each}
      </div>
    </div>
  </div>

  <!-- Performance & Visual Testing -->
  <div class="testing-panels">
    <!-- Performance Benchmarks -->
    <div class="performance-panel">
      <div class="panel-header">
        <h2 class="text-primary">Performance Benchmarks</h2>
        <div class="performance-summary">
          <span class="perf-passed text-success">{performanceMetrics.filter(m => getPerformanceStatus(m) === 'passed').length} passed</span>
          <span class="perf-failed text-error">{performanceMetrics.filter(m => getPerformanceStatus(m) === 'failed').length} failed</span>
        </div>
      </div>
      
      <div class="performance-metrics">
        {#each performanceMetrics as metric}
          <div class="metric-item card bg-surface border" class:failed={getPerformanceStatus(metric) === 'failed'}>
            <div class="metric-header">
              <div class="metric-name text-primary">{metric.test}</div>
              <div class="metric-status status-{getPerformanceStatus(metric)}">
                {getPerformanceStatus(metric)}
              </div>
            </div>
            <div class="metric-values">
              <div class="metric-actual">
                {metric.actual}ms
                <span class="metric-trend" class:positive={metric.trend.startsWith('-')} class:negative={metric.trend.startsWith('+')}>
                  {metric.trend}
                </span>
              </div>
              <div class="metric-threshold">
                Threshold: {metric.threshold}ms
              </div>
            </div>
          </div>
        {/each}
      </div>
    </div>

    <!-- Visual Regression -->
    <div class="visual-panel">
      <div class="panel-header">
        <h2 class="text-primary">Visual Regression Tests</h2>
        <div class="visual-controls">
          <button class="visual-btn">📷 Capture Baselines</button>
          <button class="visual-btn">🔍 Run Visual Tests</button>
        </div>
      </div>
      
      <div class="visual-tests">
        {#each visualTests as test}
          <div class="visual-item card bg-surface border" class:failed={test.status === 'failed'}>
            <div class="visual-header">
              <div class="visual-info">
                <div class="visual-name text-primary">{test.name}</div>
                <div class="visual-meta text-secondary">{test.browser} • {test.viewport}</div>
              </div>
              <div class="visual-status status-{test.status}">
                {test.status}
              </div>
            </div>
            
            <div class="visual-details">
              <div class="diff-percentage" class:warning={test.diff > 1} class:error={test.diff > 2}>
                {test.diff}% difference
              </div>
              {#if test.status === 'failed'}
                <button class="compare-btn">👁️ View Comparison</button>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    </div>
  </div>

  <!-- Test History Chart -->
  <div class="history-panel">
    <div class="panel-header">
      <h2>Test History (Last 7 Days)</h2>
      <div class="history-stats">
        <span class="trend-info text-success">Pass rate trending up 📈</span>
      </div>
    </div>
    
    <div class="history-chart">
      <div class="chart-bars">
        {#each testHistory as day, index}
          <div class="day-bar">
            <div class="bar-container">
              <div 
                class="bar-passed" 
                style:height="{(day.passed / day.total) * 100}%"
                title="{day.passed} passed"
              ></div>
              <div 
                class="bar-failed" 
                style:height="{(day.failed / day.total) * 100}%"
                title="{day.failed} failed"
              ></div>
            </div>
            <div class="day-label">{day.date.split('-')[2]}</div>
            <div class="day-stats">
              <div class="day-passed">{day.passed}</div>
              <div class="day-failed">{day.failed}</div>
            </div>
          </div>
        {/each}
      </div>
    </div>
  </div>

  <!-- Test Output Panel -->
  {#if showOutput}
    <div class="output-panel">
      <div class="output-header">
        <h3 class="text-primary">Test Output</h3>
        <button class="close-output" on:click={() => showOutput = false}>×</button>
      </div>
      <div class="output-content">
        <pre>{testOutput}</pre>
        {#if runningTests}
          <div class="loading-indicator">
            <div class="spinner"></div>
            <span>Running tests...</span>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .testing-quality-container {
    padding: var(--spacing-xl);
    min-height: 100vh;
    background: var(--color-background);
  }

  .testing-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-xl);
    padding: var(--spacing-lg);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
  }

  .testing-header h1 {
    font-size: 1.75rem;
    font-weight: 600;
    margin: 0;
  }

  .header-controls {
    display: flex;
    gap: 0.75rem;
  }

  .run-all-btn {
    padding: 0.5rem 1rem;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .run-all-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  /* Suite Overview */
  .suite-overview {
    margin-bottom: var(--spacing-xl);
  }

  .overview-stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
    gap: var(--spacing-lg);
  }

  .suite-card {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--spacing-lg);
    transition: all var(--transition-base);
  }

  .suite-card.running {
    border-color: var(--accent-color);
  }

  .suite-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 1rem;
  }

  .suite-info h3 {
    margin: 0 0 0.25rem 0;
    font-size: 1rem;
    font-weight: 600;
  }

  .suite-meta {
    font-size: 0.75rem;
  }

  .run-suite-btn {
    padding: 0.375rem 0.625rem;
    border-radius: 0.25rem;
    cursor: pointer;
    font-size: 0.75rem;
  }

  .suite-results {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .result-stats {
    display: flex;
    gap: 1rem;
  }

  .stat {
    text-align: center;
  }

  .stat-value {
    font-size: 1.125rem;
    font-weight: 700;
    margin-bottom: 0.125rem;
  }

  .stat-label {
    font-size: 0.625rem;
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .stat.passed .stat-value {
    color: var(--success-color);
  }

  .stat.failed .stat-value {
    color: var(--error-color);
  }

  .stat.skipped .stat-value {
    color: var(--warning-color);
  }

  .pass-rate {
    text-align: right;
  }

  .rate-value {
    font-size: 1.125rem;
    font-weight: 700;
    font-family: var(--font-mono);
  }

  .rate-label {
    font-size: 0.625rem;
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .coverage-info {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding-top: 0.75rem;
    border-top: 1px solid var(--border-color);
  }

  .coverage-label {
    font-size: 0.75rem;
    min-width: 5rem;
  }

  .coverage-bar {
    flex: 1;
    height: 6px;
    border-radius: 0.1875rem;
    overflow: hidden;
    background-color: var(--bg-surface-3);
  }

  .coverage-fill {
    height: 100%;
    transition: width 0.3s ease;
    background-color: var(--success-color);
  }

  .coverage-value {
    font-size: 0.75rem;
    font-weight: 600;
    font-family: var(--font-mono);
    min-width: 2.5rem;
    text-align: right;
  }

  /* Main Content */
  .testing-content {
    display: grid;
    grid-template-columns: 1fr 420px;
    gap: var(--spacing-xl);
    margin-bottom: var(--spacing-xl);
  }

  .test-results-panel,
  .coverage-panel {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border-color);
  }

  .panel-header h2 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }

  .result-filters {
    display: flex;
    gap: 0.5rem;
  }

  .filter-select {
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
  }

  .results-list {
    max-height: 400px;
    overflow-y: auto;
    padding: 1rem;
  }

  .result-item {
    border-radius: 0.375rem;
    padding: 0.75rem;
    margin-bottom: 0.5rem;
    transition: all 0.2s ease;
  }

  .result-item:hover {
    background-color: var(--hover-bg);
  }

  .result-item.failed {
    border-color: var(--error-color);
  }

  .result-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
  }

  .result-name {
    font-size: 0.8125rem;
    font-weight: 500;
    margin-bottom: 0.25rem;
  }

  .result-file {
    font-size: 0.6875rem;
    font-family: var(--font-mono);
  }

  .result-meta {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 0.125rem;
  }

  .result-status {
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
  }

  .result-duration {
    font-size: 0.625rem;
    font-family: var(--font-mono);
  }

  .result-error {
    margin-top: 0.5rem;
    padding: 0.5rem;
    border-radius: 0.25rem;
    background-color: var(--error-bg);
    border: 1px solid var(--error-color);
  }

  .error-message {
    font-size: 0.6875rem;
    font-family: var(--font-mono);
    line-height: 1.4;
    color: var(--error-color);
  }

  .coverage-overall {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .overall-value {
    font-size: 1.25rem;
    font-weight: 700;
    font-family: var(--font-mono);
  }

  .overall-label {
    font-size: 0.75rem;
  }

  .coverage-modules {
    padding: 1rem;
  }

  .module-item {
    margin-bottom: 1rem;
  }

  .module-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.375rem;
  }

  .module-name {
    font-size: 0.8125rem;
    font-weight: 500;
  }

  .module-coverage {
    font-size: 0.8125rem;
    font-weight: 600;
    font-family: var(--font-mono);
  }

  .module-bar {
    width: 100%;
    height: 6px;
    border-radius: 0.1875rem;
    overflow: hidden;
    margin-bottom: 0.375rem;
    background-color: var(--bg-surface-3);
  }

  .module-fill {
    height: 100%;
    transition: width 0.3s ease;
    background-color: var(--success-color);
  }

  .module-details {
    display: flex;
    justify-content: space-between;
    font-size: 0.6875rem;
  }

  /* Performance & Visual */
  .testing-panels {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--spacing-xl);
    margin-bottom: var(--spacing-xl);
  }

  .performance-panel,
  .visual-panel {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }

  .performance-summary {
    display: flex;
    gap: 0.75rem;
    font-size: 0.75rem;
  }

  .perf-passed {
    font-weight: 600;
  }

  .perf-failed {
    font-weight: 600;
  }

  .performance-metrics,
  .visual-tests {
    padding: 16px;
    max-height: 300px;
    overflow-y: auto;
  }

  .metric-item,
  .visual-item {
    border: 1px solid;
    border-radius: 6px;
    padding: 12px;
    margin-bottom: 8px;
  }

  .metric-item.failed,
  .visual-item.failed {
  }

  .metric-header,
  .visual-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }

  .metric-name,
  .visual-name {
    font-size: 13px;
    font-weight: 500;
  }

  .visual-meta {
    font-size: 11px;
  }

  .metric-status,
  .visual-status {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
  }

  .metric-values {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .metric-actual {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 600;
    font-family: monospace;
  }

  .metric-trend {
    font-size: 10px;
    padding: 2px 4px;
    border-radius: 3px;
  }

  .metric-trend.positive {
  }

  .metric-trend.negative {
  }

  .metric-threshold {
    font-size: 11px;
    font-family: monospace;
  }

  .visual-controls {
    display: flex;
    gap: 8px;
  }

  .visual-btn {
    border: 1px solid;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 11px;
    cursor: pointer;
  }

  .visual-btn:hover {
  }

  .visual-details {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .diff-percentage {
    font-size: 12px;
    font-weight: 600;
    font-family: monospace;
  }

  .diff-percentage.warning {
  }

  .diff-percentage.error {
  }

  .compare-btn {
    border: none;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 10px;
    cursor: pointer;
  }

  .compare-btn:hover {
  }

  /* History Chart */
  .history-panel {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-xl);
    overflow: hidden;
    margin-bottom: var(--spacing-xl);
  }

  .history-stats {
    font-size: 12px;
    font-weight: 600;
  }

  .history-chart {
    padding: 20px;
  }

  .chart-bars {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    height: 120px;
    gap: 8px;
  }

  .day-bar {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }

  .bar-container {
    width: 100%;
    height: 80px;
    border-radius: 4px;
    display: flex;
    flex-direction: column-reverse;
    overflow: hidden;
  }

  .bar-passed {
    transition: height 0.5s ease;
  }

  .bar-failed {
    transition: height 0.5s ease;
  }

  .day-label {
    font-size: 11px;
    font-weight: 500;
  }

  .day-stats {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }

  .day-passed {
    font-size: 10px;
    font-weight: 600;
  }

  .day-failed {
    font-size: 10px;
    font-weight: 600;
  }

  /* Output Panel */
  .output-panel {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    height: 300px;
    border: 1px solid;
    border-bottom: none;
    z-index: 1000;
  }

  .output-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 20px;
    border-bottom: 1px solid;
  }

  .output-header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }

  .close-output {
    background: none;
    border: none;
    font-size: 18px;
    cursor: pointer;
    padding: 0;
  }

  .close-output:hover {
  }

  .output-content {
    padding: 16px 20px;
    height: calc(100% - 60px);
    overflow-y: auto;
  }

  .output-content pre {
    font-family: monospace;
    font-size: 13px;
    line-height: 1.4;
    margin: 0;
    white-space: pre-wrap;
  }

  .loading-indicator {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 12px;
    font-size: 12px;
  }

  .spinner {
    width: 12px;
    height: 12px;
    border: 2px solid;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }

  /* Test Status Colors */
  .status-passed { color: var(--color-success); }
  .status-failed { color: var(--color-error); }
  .status-skipped { color: var(--color-warning); }
  .status-running { color: var(--color-info); }

  /* Coverage Colors */
  .coverage-good { color: var(--color-success); }
  .coverage-medium { color: var(--color-warning); }
  .coverage-poor { color: var(--color-error); }

  /* Coverage Fill Colors */
  .coverage-fill.coverage-good { background-color: var(--color-success); }
  .coverage-fill.coverage-medium { background-color: var(--color-warning); }
  .coverage-fill.coverage-poor { background-color: var(--color-error); }

  /* Coverage Bar Colors */
  .coverage-bar.coverage-good { background-color: var(--color-success); }
  .coverage-bar.coverage-medium { background-color: var(--color-warning); }
  .coverage-bar.coverage-poor { background-color: var(--color-error); }
</style>