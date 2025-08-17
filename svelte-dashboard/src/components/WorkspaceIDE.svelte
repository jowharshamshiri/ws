<script>
  import { onMount } from 'svelte';
  
  let files = [];
  let selectedFile = null;
  let openFiles = [];
  let activeFileIndex = 0;
  let fileContent = '';
  let searchQuery = '';
  let gitStatus = [];
  let terminalOutput = '';
  let showTerminal = false;
  let aiSuggestions = [];
  let showAIPanel = false;

  // Sample file structure
  onMount(async () => {
    files = [
      {
        name: 'src',
        type: 'folder',
        expanded: true,
        children: [
          { name: 'main.rs', type: 'file', status: 'M', language: 'rust' },
          { name: 'lib.rs', type: 'file', status: '', language: 'rust' },
          {
            name: 'components',
            type: 'folder',
            expanded: false,
            children: [
              { name: 'header.rs', type: 'file', status: 'M', language: 'rust' },
              { name: 'sidebar.rs', type: 'file', status: 'A', language: 'rust' }
            ]
          }
        ]
      },
      {
        name: 'tests',
        type: 'folder',
        expanded: false,
        children: [
          { name: 'integration.rs', type: 'file', status: '', language: 'rust' }
        ]
      },
      { name: 'Cargo.toml', type: 'file', status: 'M', language: 'toml' },
      { name: 'README.md', type: 'file', status: '', language: 'markdown' }
    ];

    gitStatus = [
      { file: 'src/main.rs', status: 'M' },
      { file: 'src/components/header.rs', status: 'M' },
      { file: 'src/components/sidebar.rs', status: 'A' },
      { file: 'Cargo.toml', status: 'M' }
    ];

    aiSuggestions = [
      {
        type: 'code_completion',
        content: 'Consider adding error handling for this function',
        line: 42,
        confidence: 0.85
      },
      {
        type: 'refactor',
        content: 'This function could be simplified using iterator combinators',
        line: 28,
        confidence: 0.72
      }
    ];
  });

  function toggleFolder(folder) {
    folder.expanded = !folder.expanded;
    files = [...files];
  }

  function openFile(file) {
    if (file.type === 'folder') {
      toggleFolder(file);
      return;
    }

    selectedFile = file;
    
    if (!openFiles.find(f => f.name === file.name)) {
      openFiles = [...openFiles, file];
    }
    
    activeFileIndex = openFiles.findIndex(f => f.name === file.name);
    loadFileContent(file);
  }

  function closeFile(index) {
    openFiles.splice(index, 1);
    openFiles = [...openFiles];
    
    if (activeFileIndex >= openFiles.length) {
      activeFileIndex = openFiles.length - 1;
    }
    
    if (openFiles.length > 0 && activeFileIndex >= 0) {
      selectedFile = openFiles[activeFileIndex];
      loadFileContent(selectedFile);
    } else {
      selectedFile = null;
      fileContent = '';
    }
  }

  function switchTab(index) {
    activeFileIndex = index;
    selectedFile = openFiles[index];
    loadFileContent(selectedFile);
  }

  function loadFileContent(file) {
    // Simulate loading file content based on type
    switch (file.name) {
      case 'main.rs':
        fileContent = `use std::collections::HashMap;

fn main() {
    println!("Hello, workspace!");
    
    let mut config = HashMap::new();
    config.insert("version", "1.0.0");
    config.insert("name", "workspace");
    
    process_config(&config);
}

fn process_config(config: &HashMap<&str, &str>) {
    for (key, value) in config {
        println!("{}: {}", key, value);
    }
}`;
        break;
      case 'Cargo.toml':
        fileContent = `[package]
name = "workspace"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
clap = { version = "4.0", features = ["derive"] }`;
        break;
      default:
        fileContent = `// ${file.name}
// File content would be loaded here
`;
    }
  }

  function runCode() {
    terminalOutput = 'Running cargo run...\n   Compiling workspace v0.1.0\n    Finished dev [unoptimized + debuginfo] target(s) in 1.23s\n     Running `target/debug/workspace`\nHello, workspace!\nversion: 1.0.0\nname: workspace';
    showTerminal = true;
  }

  function debugCode() {
    terminalOutput = 'Starting debugger...\nBreakpoint set at src/main.rs:4\nDebugger ready - use step/continue controls';
    showTerminal = true;
  }

  function runTests() {
    terminalOutput = 'Running cargo test...\n   Compiling workspace v0.1.0\nrunning 3 tests\ntest test_config ... ok\ntest test_process ... ok\ntest test_main ... ok\n\ntest result: ok. 3 passed; 0 failed; 0 ignored; 0 measured';
    showTerminal = true;
  }

  function toggleAIPanel() {
    showAIPanel = !showAIPanel;
  }

  function getFileIcon(file) {
    if (file.type === 'folder') {
      return file.expanded ? '=�' : '=�';
    }
    
    switch (file.language) {
      case 'rust': return '>�';
      case 'javascript': return '=�';
      case 'html': return '<';
      case 'css': return '<�';
      case 'markdown': return '=�';
      case 'toml': return '�';
      default: return '=�';
    }
  }

  function getStatusColor(status) {
    switch (status) {
      case 'M': return 'var(--color-warning)'; // Modified - orange
      case 'A': return 'var(--color-success)'; // Added - green
      case 'D': return 'var(--color-error)'; // Deleted - red
      default: return 'transparent';
    }
  }
</script>

<div class="workspace-ide-container card bg-surface">
  <!-- File Explorer Sidebar -->
  <div class="file-explorer card bg-surface-2">
    <div class="explorer-header">
      <h3 class="text-primary">Explorer</h3>
      <div class="explorer-actions">
        <button class="btn-secondary action-btn" title="New File">+</button>
        <button class="action-btn" title="New Folder">=�</button>
        <button class="action-btn" title="Refresh">=</button>
      </div>
    </div>
    
    <div class="search-box">
      <input 
        type="text" 
        placeholder="Search files..."
        bind:value={searchQuery}
        class="search-input bg-surface border rounded-md"
      >
    </div>

    <div class="file-tree">
      {#each files as file}
        <div class="file-item" class:folder={file.type === 'folder'}>
          <button 
            class="file-button"
            on:click={() => openFile(file)}
            class:selected={selectedFile === file}
          >
            <span class="file-icon">{getFileIcon(file)}</span>
            <span class="file-name">{file.name}</span>
            {#if file.status}
              <span 
                class="status-indicator status-{file.status}"
                title="Modified"
              ></span>
            {/if}
          </button>
          
          {#if file.type === 'folder' && file.expanded && file.children}
            <div class="folder-children">
              {#each file.children as child}
                <button 
                  class="file-button child"
                  on:click={() => openFile(child)}
                  class:selected={selectedFile === child}
                >
                  <span class="file-icon">{getFileIcon(child)}</span>
                  <span class="file-name">{child.name}</span>
                  {#if child.status}
                    <span 
                      class="status-indicator status-{child.status}"
                      title="Modified"
                    ></span>
                  {/if}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </div>

  <!-- Main Editor Area -->
  <div class="editor-area">
    <!-- Action Toolbar -->
    <div class="toolbar">
      <div class="toolbar-group">
        <button class="btn-primary toolbar-btn" on:click={runCode}>
          � Run
        </button>
        <button class="btn-secondary toolbar-btn" on:click={debugCode}>
          = Debug
        </button>
        <button class="btn-secondary toolbar-btn" on:click={runTests}>
          >� Test
        </button>
      </div>
      
      <div class="toolbar-group">
        <button class="btn-secondary toolbar-btn" on:click={toggleAIPanel} class:active={showAIPanel}>
          > AI Assist
        </button>
        <button class="toolbar-btn" title="Format Code">
          =� Format
        </button>
        <button class="toolbar-btn" title="Find in Files">
          = Search
        </button>
      </div>
    </div>

    <!-- File Tabs -->
    {#if openFiles.length > 0}
      <div class="file-tabs">
        {#each openFiles as file, index}
          <div 
            class="file-tab"
            class:active={index === activeFileIndex}
            on:click={() => switchTab(index)}
          >
            <span class="tab-icon">{getFileIcon(file)}</span>
            <span class="tab-name">{file.name}</span>
            {#if file.status}
              <span class="tab-status status-{file.status}">●</span>
            {/if}
            <button class="tab-close" on:click|stopPropagation={() => closeFile(index)}>�</button>
          </div>
        {/each}
      </div>
    {/if}

    <!-- Code Editor -->
    <div class="editor-container">
      {#if selectedFile}
        <div class="code-editor">
          <div class="line-numbers">
            {#each fileContent.split('\n') as line, index}
              <div class="line-number">{index + 1}</div>
            {/each}
          </div>
          <textarea
            class="code-textarea"
            bind:value={fileContent}
            spellcheck="false"
            placeholder="Select a file to edit..."
          ></textarea>
        </div>
      {:else}
        <div class="empty-editor">
          <div class="empty-state">
            <h3 class="text-primary">Welcome to Workspace IDE</h3>
            <p class="text-secondary">Select a file from the explorer to start editing</p>
            <div class="quick-actions">
              <button class="quick-btn">=� New File</button>
              <button class="quick-btn">=� Open Folder</button>
              <button class="quick-btn">= Quick Open</button>
            </div>
          </div>
        </div>
      {/if}
    </div>

    <!-- Terminal Panel -->
    {#if showTerminal}
      <div class="terminal-panel">
        <div class="terminal-header">
          <div class="terminal-tabs">
            <div class="terminal-tab active">Terminal</div>
            <div class="terminal-tab">Output</div>
            <div class="terminal-tab">Debug Console</div>
          </div>
          <button class="terminal-close" on:click={() => showTerminal = false}>�</button>
        </div>
        <div class="terminal-content">
          <pre>{terminalOutput}</pre>
          <div class="terminal-input">
            <span class="prompt">$ </span>
            <input type="text" class="command-input" placeholder="Type a command...">
          </div>
        </div>
      </div>
    {/if}
  </div>

  <!-- AI Assistant Panel -->
  {#if showAIPanel}
    <div class="ai-panel card bg-surface-2">
      <div class="ai-header">
        <h3>> AI Assistant</h3>
        <button class="ai-close" on:click={toggleAIPanel}>�</button>
      </div>
      
      <div class="ai-suggestions">
        <h4 class="text-secondary">Suggestions</h4>
        {#each aiSuggestions as suggestion}
          <div class="suggestion-item">
            <div class="suggestion-type">{suggestion.type}</div>
            <div class="suggestion-content">{suggestion.content}</div>
            <div class="suggestion-meta">
              Line {suggestion.line} " {Math.round(suggestion.confidence * 100)}% confidence
            </div>
            <div class="suggestion-actions">
              <button class="suggestion-btn">Apply</button>
              <button class="suggestion-btn secondary">Dismiss</button>
            </div>
          </div>
        {/each}
      </div>

      <div class="ai-chat">
        <h4 class="text-secondary">Ask AI</h4>
        <div class="chat-input">
          <textarea 
            placeholder="Ask about your code, request refactoring suggestions, or get help with implementation..."
            rows="3"
          ></textarea>
          <button class="send-btn">Send</button>
        </div>
      </div>
    </div>
  {/if}
</div>

