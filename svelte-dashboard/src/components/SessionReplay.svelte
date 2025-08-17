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
  <div class="card">
    <div class="card__header">
      <h1 class="card__title">Session Replay</h1>
      
      <div class="session-selector">
        <label class="form__label">Select Session:</label>
        <select on:change={(e) => selectSession(sessions.find(s => s.id === e.target.value))} class="form__select">
          <option value="">Choose a session...</option>
          {#each sessions as session}
            <option value={session.id} selected={selectedSession?.id === session.id}>
              {session.title} - {session.date}
            </option>
          {/each}
        </select>
      </div>
    </div>
  </div>

  {#if selectedSession}
    <div class="replay-interface">
      <!-- Left Panel - AI Reasoning -->
      <div class="card panel panel--reasoning">
        <h2 class="card__title">AI Reasoning</h2>
        
        <div class="current-reasoning">
          {#if currentEvent}
            <div class="card card--interactive card--active">
              <div class="step-icon">{getEventIcon(currentEvent.type)}</div>
              <div class="step-content">
                <div class="step-title">{currentEvent.description}</div>
                <div class="step-reasoning">{currentEvent.reasoning}</div>
                <div class="step-time">{formatTime(currentEvent.time)}</div>
              </div>
            </div>
          {/if}
        </div>

        <div class="reasoning-timeline">
          <h3 class="heading heading--small">Session Steps</h3>
          {#each selectedTimeline as event}
            <div 
              class="card card--interactive timeline-step" 
              class:card--active={currentEvent?.time === event.time}
              class:timeline-step--completed={currentTime > event.time}
              on:click={() => setTime(event.time)}
            >
              <div class="step-icon">{getEventIcon(event.type)}</div>
              <div class="step-content">
                <div class="step-title">{event.description}</div>
                <div class="step-time">{formatTime(event.time)}</div>
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

