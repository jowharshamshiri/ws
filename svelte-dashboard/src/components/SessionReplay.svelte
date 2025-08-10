<script>
  import { onMount } from 'svelte';
  import { sessionsStore } from '../stores.js';
  
  let sessions = [];
  let selectedSession = null;
  let currentTime = 0;
  let isPlaying = false;
  let playbackSpeed = 1;
  
  // Sample session data
  const sampleSessions = [
    {
      id: 'session-001',
      title: 'ADE Implementation Session',
      date: '2025-08-08',
      duration: '45m 23s',
      filesModified: 12,
      featuresChanged: 5,
      timeline: [
        { time: 0, type: 'start', description: 'Session initialized', reasoning: 'Loading project context and features analysis' },
        { time: 30, type: 'code', description: 'Created Svelte project structure', reasoning: 'Setting up modern frontend framework for ADE interface' },
        { time: 120, type: 'code', description: 'Implemented Header component', reasoning: 'Navigation system needed for multi-section ADE interface' },
        { time: 180, type: 'api', description: 'Added API service layer', reasoning: 'Connecting frontend to existing backend endpoints' },
        { time: 240, type: 'test', description: 'Built and tested components', reasoning: 'Ensuring components work correctly before proceeding' }
      ]
    },
    {
      id: 'session-002', 
      title: 'Feature Management Implementation',
      date: '2025-08-07',
      duration: '32m 15s',
      filesModified: 8,
      featuresChanged: 3,
      timeline: [
        { time: 0, type: 'start', description: 'Context loaded', reasoning: 'Continuing from previous feature work' },
        { time: 45, type: 'code', description: 'Added feature state validation', reasoning: 'Ensuring data integrity in feature transitions' },
        { time: 120, type: 'code', description: 'Implemented filtering system', reasoning: 'Users need to find features quickly in large projects' }
      ]
    }
  ];

  $: sessions = $sessionsStore.length > 0 ? $sessionsStore : sampleSessions;
  $: selectedTimeline = selectedSession?.timeline || [];
  $: currentEvent = selectedTimeline.find(event => 
    event.time <= currentTime && 
    (selectedTimeline.find(e => e.time > currentTime)?.time || Infinity) > event.time
  );

  function selectSession(session) {
    selectedSession = session;
    currentTime = 0;
    isPlaying = false;
  }

  function playPause() {
    isPlaying = !isPlaying;
    if (isPlaying) {
      startPlayback();
    }
  }

  function startPlayback() {
    if (!isPlaying || !selectedSession) return;
    
    const interval = setInterval(() => {
      if (!isPlaying) {
        clearInterval(interval);
        return;
      }
      
      currentTime += playbackSpeed;
      const maxTime = Math.max(...selectedTimeline.map(e => e.time));
      
      if (currentTime > maxTime + 30) {
        isPlaying = false;
        clearInterval(interval);
      }
    }, 100);
  }

  function setTime(time) {
    currentTime = time;
  }

  function getEventIcon(type) {
    switch (type) {
      case 'start': return '🚀';
      case 'code': return '💻';
      case 'api': return '🔗';
      case 'test': return '🧪';
      case 'error': return '🚨';
      default: return '📝';
    }
  }

  function formatTime(seconds) {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  onMount(() => {
    if (sessions.length > 0 && !selectedSession) {
      selectSession(sessions[0]);
    }
  });
</script>

<div class="session-replay-container">
  <div class="session-header card">
    <h1 class="text-primary">Session Replay</h1>
    
    <div class="session-selector">
      <label class="text-secondary">Select Session:</label>
      <select on:change={(e) => selectSession(sessions.find(s => s.id === e.target.value))} class="bg-surface border rounded-md">
        <option value="">Choose a session...</option>
        {#each sessions as session}
          <option value={session.id} selected={selectedSession?.id === session.id}>
            {session.title} - {session.date}
          </option>
        {/each}
      </select>
    </div>
  </div>

  {#if selectedSession}
    <div class="replay-interface">
      <!-- Left Panel - AI Reasoning -->
      <div class="reasoning-panel card bg-surface">
        <h2 class="text-primary">AI Reasoning</h2>
        
        <div class="current-reasoning">
          {#if currentEvent}
            <div class="reasoning-step card bg-surface-2 border-primary">
              <div class="step-icon">{getEventIcon(currentEvent.type)}</div>
              <div class="step-content">
                <div class="step-title text-primary">{currentEvent.description}</div>
                <div class="step-reasoning text-secondary">{currentEvent.reasoning}</div>
                <div class="step-time text-tertiary">{formatTime(currentEvent.time)}</div>
              </div>
            </div>
          {/if}
        </div>

        <div class="reasoning-timeline">
          <h3 class="text-primary">Session Steps</h3>
          {#each selectedTimeline as event}
            <div 
              class="reasoning-step card bg-surface-2" 
              class:active={currentEvent?.time === event.time}
              class:completed={currentTime > event.time}
              class:border-primary={currentEvent?.time === event.time}
              on:click={() => setTime(event.time)}
            >
              <div class="step-icon">{getEventIcon(event.type)}</div>
              <div class="step-content">
                <div class="step-title text-primary">{event.description}</div>
                <div class="step-time text-tertiary">{formatTime(event.time)}</div>
              </div>
            </div>
          {/each}
        </div>
      </div>

      <!-- Center Panel - Code Changes -->
      <div class="code-panel card bg-surface">
        <div class="panel-header">
          <h2 class="text-primary">Code Changes</h2>
          <div class="file-tabs">
            <div class="tab card bg-surface-2 text-primary border-primary">App.svelte</div>
            <div class="tab card bg-surface-3 text-secondary">Header.svelte</div>
            <div class="tab card bg-surface-3 text-secondary">stores.js</div>
          </div>
        </div>
        
        <div class="code-viewer">
          <div class="code-content">
            {#if currentEvent}
              <div class="diff-view">
                <div class="diff-line added">+ import Header from './components/Header.svelte';</div>
                <div class="diff-line added">+ import Overview from './components/Overview.svelte';</div>
                <div class="diff-line context">  import &#123;&#123; projectStore, featuresStore &#125;&#125; from './stores.js';</div>
                <div class="diff-line removed">- &lt;div class="old-dashboard"&gt;&lt;/div&gt;</div>
                <div class="diff-line added">+ &lt;div class="ade-app"&gt;&lt;/div&gt;</div>
                <div class="diff-line added">+   &lt;Header bind:currentView /&gt;</div>
                <div class="diff-line context">  </div>
              </div>
            {:else}
              <div class="placeholder text-tertiary">
                Select a session event to view code changes
              </div>
            {/if}
          </div>
        </div>
      </div>

      <!-- Right Panel - Tool Usage -->
      <div class="tools-panel card bg-surface">
        <h2 class="text-primary">Tool Usage</h2>
        
        <div class="tool-timeline">
          {#each selectedTimeline as event}
            <div 
              class="tool-item" 
              class:active={currentEvent?.time === event.time}
              class:completed={currentTime > event.time}
            >
              <div class="tool-badge" class:used={currentTime >= event.time}>
                {getEventIcon(event.type)}
              </div>
              <div class="tool-info">
                <div class="tool-name">{event.type.toUpperCase()}</div>
                <div class="tool-time">{formatTime(event.time)}</div>
              </div>
            </div>
          {/each}
        </div>

        <div class="session-stats card bg-surface-2">
          <h3 class="text-primary">Session Statistics</h3>
          <div class="stat-item">
            <span class="stat-label text-secondary">Duration:</span>
            <span class="stat-value text-primary">{selectedSession.duration}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label text-secondary">Files Modified:</span>
            <span class="stat-value text-primary">{selectedSession.filesModified}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label text-secondary">Features Changed:</span>
            <span class="stat-value text-primary">{selectedSession.featuresChanged}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Playback Controls -->
    <div class="playback-controls card bg-surface">
      <div class="timeline-scrubber">
        <input 
          type="range" 
          min="0" 
          max={Math.max(...selectedTimeline.map(e => e.time)) + 30}
          bind:value={currentTime}
          class="timeline-slider"
        />
        <div class="timeline-markers">
          {#each selectedTimeline as event}
            <div 
              class="timeline-marker" 
              on:click={() => setTime(event.time)}
              style:left="{(event.time / (Math.max(...selectedTimeline.map(e => e.time)) + 30)) * 100}%"
            >
              <div class="marker-tooltip">{event.description}</div>
            </div>
          {/each}
        </div>
      </div>
      
      <div class="control-buttons">
        <button class="control-btn" on:click={() => setTime(0)}>⏮</button>
        <button class="control-btn play-btn" on:click={playPause}>
          {isPlaying ? '⏸' : '▶️'}
        </button>
        <button class="control-btn" on:click={() => setTime(Math.max(...selectedTimeline.map(e => e.time)))}>⏭</button>
        
        <div class="time-display">
          {formatTime(currentTime)} / {formatTime(Math.max(...selectedTimeline.map(e => e.time)))}
        </div>
        
        <div class="speed-control">
          <label>Speed:</label>
          <select bind:value={playbackSpeed}>
            <option value={0.5}>0.5x</option>
            <option value={1}>1x</option>
            <option value={2}>2x</option>
            <option value={4}>4x</option>
          </select>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .session-replay-container {
    padding: var(--spacing-xl);
    min-height: 100vh;
    background: var(--color-background);
  }

  .session-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--spacing-xl);
    padding: var(--spacing-lg);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
  }

  .session-selector {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .session-selector select {
    padding: var(--spacing-sm) var(--spacing-md);
    font-size: var(--font-size-sm);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    color: var(--color-text-primary);
  }

  .replay-interface {
    display: grid;
    grid-template-columns: 320px 1fr 280px;
    gap: var(--spacing-lg);
    min-height: 600px;
    margin-bottom: var(--spacing-xl);
  }

  .reasoning-panel,
  .code-panel,
  .tools-panel {
    overflow: hidden;
    border-radius: var(--radius-lg);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
  }

  .reasoning-panel h2,
  .code-panel h2,
  .tools-panel h2 {
    font-size: var(--font-size-lg);
    font-weight: 600;
    margin: 0;
    padding: var(--spacing-lg);
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface-2);
  }

  .current-reasoning {
    padding: var(--spacing-lg);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .reasoning-timeline {
    padding: var(--spacing-lg);
    overflow-y: auto;
    flex: 1;
    min-height: 0;
  }

  .reasoning-timeline h3 {
    font-size: var(--text-xs);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wide);
    margin: 0 0 var(--space-4) 0;
  }

  .reasoning-step {
    display: flex;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    border-radius: var(--radius-md);
    margin-bottom: var(--spacing-sm);
    cursor: pointer;
    transition: all var(--transition-base);
    background: var(--color-surface-2);
    border: 1px solid var(--color-border);
  }

  .reasoning-step:hover {
    background-color: var(--color-surface-3);
    border-color: var(--color-border-focus);
  }

  .reasoning-step.active {
    background-color: var(--color-surface-3);
    border-color: var(--color-primary);
  }

  .reasoning-step.completed {
    opacity: 0.7;
  }

  .step-icon {
    font-size: var(--text-base);
    min-width: var(--space-5);
  }

  .step-content {
    flex: 1;
  }

  .step-title {
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    margin-bottom: var(--space-1);
    line-height: var(--leading-tight);
  }

  .step-reasoning {
    font-size: var(--text-xs);
    line-height: var(--leading-relaxed);
    margin-bottom: var(--space-1);
  }

  .step-time {
    font-size: var(--text-xs);
    font-family: var(--font-mono);
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-4) var(--space-5);
    border-bottom: 1px solid var(--color-border);
  }

  .file-tabs {
    display: flex;
    gap: var(--space-1);
  }

  .tab {
    padding: var(--space-1-5) var(--space-3);
    border-radius: var(--radius-sm);
    font-size: var(--text-xs);
    cursor: pointer;
    transition: all var(--transition-fast) ease;
  }

  .tab:hover {
    background-color: var(--color-bg-hover);
  }

  .code-viewer {
    height: 400px;
    overflow: auto;
  }

  .code-content {
    padding: var(--spacing-xl);
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    font-size: var(--font-size-sm);
    line-height: 1.6;
  }

  .diff-view {
    display: flex;
    flex-direction: column;
    gap: var(--space-0-5);
  }

  .diff-line {
    padding: var(--space-0-5) var(--space-2);
    border-radius: var(--radius-sm);
  }

  .placeholder {
    text-align: center;
    padding: var(--space-10) var(--space-5);
    font-style: italic;
  }

  .tool-timeline {
    padding: var(--spacing-lg);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    flex: 1;
    overflow-y: auto;
  }

  .tool-item {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2);
    border-radius: var(--radius-md);
    transition: all var(--transition-fast) ease;
  }

  .tool-item.active {
    background-color: var(--color-bg-accent);
  }

  .tool-badge {
    width: var(--space-6);
    height: var(--space-6);
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--color-bg-surface-3);
    border-radius: var(--radius-full);
    font-size: var(--text-xs);
    transition: all var(--transition-fast) ease;
  }

  .tool-badge.used {
    background-color: var(--color-bg-accent);
  }

  .tool-name {
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
  }

  .tool-time {
    font-size: var(--text-xs);
    font-family: var(--font-mono);
  }

  .session-stats {
    margin-top: var(--space-5);
    padding: var(--space-4) var(--space-5);
    border-top: 1px solid var(--color-border);
  }

  .session-stats h3 {
    font-size: var(--text-xs);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wide);
    margin: 0 0 var(--space-3) 0;
  }

  .stat-item {
    display: flex;
    justify-content: space-between;
    margin-bottom: var(--space-2);
    font-size: var(--text-xs);
  }

  .playback-controls {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--spacing-lg);
  }

  .timeline-scrubber {
    position: relative;
    margin-bottom: var(--space-4);
  }

  .timeline-slider {
    width: 100%;
    height: 6px;
    background-color: var(--color-bg-surface-3);
    border-radius: var(--radius-sm);
    outline: none;
    appearance: none;
  }

  .timeline-slider::-webkit-slider-thumb {
    appearance: none;
    width: var(--space-4);
    height: var(--space-4);
    background-color: var(--color-accent);
    border-radius: var(--radius-full);
    cursor: pointer;
  }

  .timeline-markers {
    position: absolute;
    top: -2px;
    left: 0;
    right: 0;
    height: 10px;
    pointer-events: none;
  }

  .timeline-marker {
    position: absolute;
    width: 4px;
    height: 10px;
    background-color: var(--color-warning);
    cursor: pointer;
    pointer-events: all;
  }

  .control-buttons {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .time-display {
    color: var(--color-accent);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    margin-left: auto;
  }

  .speed-control {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .speed-control select {
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-sm);
    font-size: var(--text-xs);
  }
</style>