// Workspace Project Dashboard JavaScript

class Dashboard {
    constructor() {
        this.apiBase = '/api';
        this.updateInterval = 5000; // 5 seconds
        this.currentFeatureFilter = 'all';
        this.currentTaskFilter = 'all';
        this.currentDirectiveFilter = 'all';
        this.currentRelationshipFilter = 'all';
        this.currentNoteLinkFilter = 'all';
        this.currentMilestoneFilter = 'all';
        
        // Cached data
        this.milestones = null;
        
        // Git integration state
        this.currentView = 'timeline';
        this.selectedCommit = 'HEAD';
        this.selectedFile = null;
        this.monacoEditor = null;
        this.diffEditor = null;
        this.commits = [];
        this.files = [];
        
        this.init();
    }
    
    async init() {
        await this.loadInitialData();
        this.setupEventListeners();
        this.startAutoRefresh();
        
        console.log('Dashboard initialized at', new Date().toISOString());
    }
    
    async loadInitialData() {
        console.log('Starting dashboard initialization...');
        
        // Load each component independently to avoid total failure
        const loadPromises = [
            this.loadProjectStatus().catch(e => console.error('Failed to load project status:', e)),
            this.loadFeatures().catch(e => console.error('Failed to load features:', e)),
            this.loadTasks().catch(e => console.error('Failed to load tasks:', e)),
            this.loadDirectives().catch(e => console.error('Failed to load directives:', e)),
            this.loadRelationships().catch(e => console.error('Failed to load relationships:', e)),
            this.loadMilestones().catch(e => console.error('Failed to load milestones:', e)),
            this.loadNoteLinks().catch(e => console.error('Failed to load note links:', e)),
            this.loadActivity().catch(e => console.error('Failed to load activity:', e)),
            this.initializeGitIntegration().catch(e => console.error('Failed to initialize git:', e))
        ];
        
        await Promise.allSettled(loadPromises);
        this.updateLastUpdated();
        console.log('Dashboard initialization complete');
    }
    
    async loadProjectStatus() {
        try {
            const response = await fetch(`${this.apiBase}/project/status`);
            if (!response.ok) throw new Error('Failed to fetch project status');
            
            const data = await response.json();
            this.updateProjectOverview(data.project);
            this.updateFeatureMetrics(data.feature_metrics);
            this.updateTaskSummary(data.task_summary);
        } catch (error) {
            console.error('Error loading project status:', error);
        }
    }
    
    async loadFeatures() {
        try {
            const response = await fetch(`${this.apiBase}/features`);
            if (!response.ok) throw new Error('Failed to fetch features');
            
            const features = await response.json();
            this.renderFeatures(features);
        } catch (error) {
            console.error('Error loading features:', error);
        }
    }
    
    async loadTasks() {
        try {
            const response = await fetch(`${this.apiBase}/tasks`);
            if (!response.ok) throw new Error('Failed to fetch tasks');
            
            const tasks = await response.json();
            this.renderTasks(tasks);
        } catch (error) {
            console.error('Error loading tasks:', error);
        }
    }
    
    async loadDirectives() {
        try {
            const response = await fetch(`${this.apiBase}/directives`);
            if (!response.ok) throw new Error('Failed to fetch directives');
            
            const directives = await response.json();
            this.renderDirectives(directives);
        } catch (error) {
            console.error('Error loading directives:', error);
        }
    }
    
    async loadRelationships() {
        console.log('Loading relationships...');
        try {
            const response = await fetch(`${this.apiBase}/relationships`);
            if (!response.ok) throw new Error('Failed to fetch relationships');
            
            const relationships = await response.json();
            console.log(`Loaded ${relationships.length} relationships`);
            this.renderRelationships(relationships);
        } catch (error) {
            console.error('Error loading relationships:', error);
            const container = document.getElementById('relationship-list');
            if (container) {
                container.innerHTML = '<div class="loading">Failed to load relationships</div>';
            }
        }
    }
    
    async loadActivity() {
        try {
            // For now, show placeholder activity
            const activity = [
                {
                    timestamp: new Date().toISOString(),
                    activity_type: 'feature_update',
                    description: 'MCP Server Integration Foundation feature updated',
                    entity_type: 'feature',
                    entity_id: 'F0096'
                }
            ];
            this.renderActivity(activity);
        } catch (error) {
            console.error('Error loading activity:', error);
        }
    }
    
    updateProjectOverview(project) {
        document.getElementById('project-name').textContent = project.name || 'Workspace Project';
        document.getElementById('project-description').textContent = 
            project.description || 'AI-assisted development tool suite';
    }
    
    updateFeatureMetrics(metrics) {
        document.getElementById('total-features').textContent = metrics.total;
        document.getElementById('implemented-features').textContent = metrics.implemented;
        document.getElementById('tested-features').textContent = metrics.tested;
        document.getElementById('implementation-percentage').textContent = 
            `${Math.round(metrics.implementation_percentage)}%`;
        
        // Update progress bars
        document.getElementById('implementation-bar').style.width = 
            `${metrics.implementation_percentage}%`;
        document.getElementById('testing-bar').style.width = 
            `${metrics.test_coverage_percentage}%`;
    }
    
    updateTaskSummary(summary) {
        document.getElementById('pending-tasks').textContent = summary.pending;
        document.getElementById('progress-tasks').textContent = summary.in_progress;
        document.getElementById('completed-tasks').textContent = summary.completed;
        document.getElementById('blocked-tasks').textContent = summary.blocked;
    }
    
    renderFeatures(features) {
        const container = document.getElementById('feature-list');
        
        if (features.length === 0) {
            container.innerHTML = '<div class="loading">No features found</div>';
            return;
        }
        
        const filteredFeatures = this.filterFeatures(features);
        
        container.innerHTML = filteredFeatures.map(feature => `
            <div class="feature-item clickable" data-status="${feature.state}" data-entity-type="feature" data-entity-id="${feature.code}">
                <div class="title">${feature.name}</div>
                <div class="description">${feature.description}</div>
                <div class="meta">
                    <span class="feature-id">${feature.code}</span>
                    <div class="badges">
                        <span class="status-badge ${feature.state.toLowerCase()}">
                            ${this.formatStatus(feature.state)}
                        </span>
                        <span class="status-badge ${feature.test_status.toLowerCase()}">
                            ${this.formatStatus(feature.test_status)}
                        </span>
                    </div>
                </div>
            </div>
        `).join('');
        
        // Add click listeners for feature items
        container.querySelectorAll('.feature-item').forEach(item => {
            item.addEventListener('click', async () => {
                const entityId = item.dataset.entityId;
                try {
                    const response = await fetch(`${this.apiBase}/features/${entityId}`);
                    if (response.ok) {
                        const feature = await response.json();
                        this.showEntityDialog('feature', feature);
                    } else {
                        console.error('Failed to fetch feature details');
                    }
                } catch (error) {
                    console.error('Error fetching feature details:', error);
                }
            });
        });
    }
    
    renderTasks(tasks) {
        const container = document.getElementById('task-list');
        
        if (tasks.length === 0) {
            container.innerHTML = '<div class="loading">No tasks found</div>';
            return;
        }
        
        const filteredTasks = this.filterTasks(tasks);
        
        container.innerHTML = filteredTasks.map(task => `
            <div class="task-item clickable" data-status="${task.status}" data-entity-type="task" data-entity-id="${task.id}">
                <div class="title">${task.title}</div>
                <div class="description">${task.description}</div>
                <div class="meta">
                    <span class="task-id">${task.id}</span>
                    <span class="status-badge ${task.status}">
                        ${this.formatStatus(task.status)}
                    </span>
                </div>
            </div>
        `).join('');
        
        // Add click listeners for task items
        container.querySelectorAll('.task-item').forEach(item => {
            item.addEventListener('click', async () => {
                const entityId = item.dataset.entityId;
                try {
                    const response = await fetch(`${this.apiBase}/tasks/${entityId}`);
                    if (response.ok) {
                        const task = await response.json();
                        this.showEntityDialog('task', task);
                    } else {
                        console.error('Failed to fetch task details');
                    }
                } catch (error) {
                    console.error('Error fetching task details:', error);
                }
            });
        });
    }
    
    renderDirectives(directives) {
        const container = document.getElementById('directive-list');
        
        if (directives.length === 0) {
            container.innerHTML = '<div class="loading">No directives found</div>';
            return;
        }
        
        const filteredDirectives = this.filterDirectives(directives);
        
        container.innerHTML = filteredDirectives.map(directive => `
            <div class="directive-item clickable" data-category="${directive.category}" data-entity-type="directive" data-entity-id="${directive.id}">
                <div class="title">${directive.title}</div>
                <div class="description">${directive.rule}</div>
                <div class="meta">
                    <span class="directive-id">${directive.code}</span>
                    <div class="badges">
                        <span class="status-badge ${directive.category.toLowerCase()}">
                            ${directive.category}
                        </span>
                        <span class="priority-badge ${directive.priority.toLowerCase()}">
                            ${this.formatStatus(directive.priority)}
                        </span>
                        <span class="status-badge ${directive.active ? 'active' : 'inactive'}">
                            ${directive.active ? 'Active' : 'Inactive'}
                        </span>
                    </div>
                </div>
            </div>
        `).join('');
        
        // Add click listeners for directive items
        container.querySelectorAll('.directive-item').forEach(item => {
            item.addEventListener('click', async () => {
                const entityId = item.dataset.entityId;
                try {
                    const response = await fetch(`${this.apiBase}/directives/${entityId}`);
                    if (response.ok) {
                        const directive = await response.json();
                        this.showEntityDialog('directive', directive);
                    } else {
                        console.error('Failed to fetch directive details');
                    }
                } catch (error) {
                    console.error('Error fetching directive details:', error);
                }
            });
        });
    }
    
    renderRelationships(relationships) {
        console.log('Rendering relationships...');
        const container = document.getElementById('relationship-list');
        if (!container) {
            console.error('relationship-list container not found!');
            return;
        }
        
        if (relationships.length === 0) {
            container.innerHTML = '<div class="loading">No relationships found</div>';
            return;
        }
        
        const filteredRelationships = this.filterRelationships(relationships);
        
        container.innerHTML = filteredRelationships.map(relationship => `
            <div class="relationship-item clickable" data-type="${relationship.dependency_type}" data-entity-type="relationship" data-entity-id="${relationship.id}">
                <div class="relationship-header">
                    <div class="relationship-source">
                        <span class="entity-type">${relationship.from_entity_type}</span>
                        <span class="entity-id">${relationship.from_entity_id}</span>
                    </div>
                    <div class="relationship-arrow">→</div>
                    <div class="relationship-target">
                        <span class="entity-type">${relationship.to_entity_type}</span>
                        <span class="entity-id">${relationship.to_entity_id}</span>
                    </div>
                </div>
                <div class="relationship-meta">
                    <span class="relationship-type ${relationship.dependency_type.toLowerCase()}">
                        ${relationship.dependency_type}
                    </span>
                    ${relationship.resolved_at ? '<span class="status-badge resolved">Resolved</span>' : '<span class="status-badge active">Active</span>'}
                    ${relationship.description ? `<div class="description">${relationship.description}</div>` : ''}
                </div>
            </div>
        `).join('');
        
        // Add click listeners for relationship items
        container.querySelectorAll('.relationship-item').forEach(item => {
            item.addEventListener('click', async () => {
                const entityId = item.dataset.entityId;
                try {
                    const response = await fetch(`${this.apiBase}/relationships`);
                    if (response.ok) {
                        const relationships = await response.json();
                        const relationship = relationships.find(r => r.id === entityId);
                        if (relationship) {
                            this.showEntityDialog('relationship', relationship);
                        }
                    } else {
                        console.error('Failed to fetch relationship details');
                    }
                } catch (error) {
                    console.error('Error fetching relationship details:', error);
                }
            });
        });
    }
    
    async loadMilestones() {
        console.log('Loading milestones...');
        try {
            const response = await fetch(`${this.apiBase}/milestones`);
            if (!response.ok) throw new Error('Failed to fetch milestones');
            
            const milestones = await response.json();
            console.log(`Loaded ${milestones.length} milestones`);
            this.milestones = milestones; // Cache milestones
            this.renderMilestones(milestones);
        } catch (error) {
            console.error('Error loading milestones:', error);
            const container = document.getElementById('milestone-list');
            if (container) {
                container.innerHTML = '<div class="loading">Failed to load milestones</div>';
            }
        }
    }
    
    renderMilestones(milestones) {
        console.log('Rendering milestones...', milestones.length, 'milestones, filter:', this.currentMilestoneFilter);
        const container = document.getElementById('milestone-list');
        if (!container) {
            console.error('milestone-list container not found!');
            return;
        }
        
        if (milestones.length === 0) {
            container.innerHTML = '<div class="loading">No milestones found</div>';
            return;
        }
        
        const filteredMilestones = this.filterMilestones(milestones);
        
        container.innerHTML = filteredMilestones.map(milestone => `
            <div class="milestone-item clickable" data-status="${milestone.status}" data-entity-type="milestone" data-entity-id="${milestone.id}">
                <div class="milestone-header">
                    <div class="milestone-title">${milestone.title}</div>
                    <div class="milestone-status">
                        <span class="status-badge ${milestone.status.toLowerCase()}">
                            ${this.formatMilestoneStatus(milestone.status)}
                        </span>
                    </div>
                </div>
                <div class="milestone-description">${milestone.description}</div>
                <div class="milestone-meta">
                    <div class="completion-bar">
                        <div class="completion-fill" style="width: ${milestone.completion_percentage}%"></div>
                        <span class="completion-text">${Math.round(milestone.completion_percentage)}%</span>
                    </div>
                    ${milestone.target_date ? `<div class="target-date">Target: ${this.formatDateTime(milestone.target_date)}</div>` : ''}
                    ${milestone.achieved_date ? `<div class="achieved-date">Achieved: ${this.formatDateTime(milestone.achieved_date)}</div>` : ''}
                </div>
            </div>
        `).join('');
        
        // Add click listeners for milestone items
        container.querySelectorAll('.milestone-item').forEach(item => {
            item.addEventListener('click', async () => {
                const entityId = item.dataset.entityId;
                try {
                    const response = await fetch(`${this.apiBase}/milestones/${entityId}`);
                    if (response.ok) {
                        const milestone = await response.json();
                        this.showEntityDialog('milestone', milestone);
                    } else {
                        console.error('Failed to fetch milestone details');
                    }
                } catch (error) {
                    console.error('Error fetching milestone details:', error);
                }
            });
        });

        // Milestone filters are set up in setupEventListeners()
    }
    
    filterMilestones(milestones) {
        if (this.currentMilestoneFilter === 'all') return milestones;
        return milestones.filter(milestone => milestone.status === this.currentMilestoneFilter);
    }
    
    formatMilestoneStatus(status) {
        const statusMap = {
            'planned': 'Planned',
            'in_progress': 'In Progress', 
            'achieved': 'Achieved',
            'missed': 'Missed'
        };
        return statusMap[status] || status;
    }
    
    setupMilestoneFilters() {
        const filterBtns = document.querySelectorAll('.milestones .filter-btn');
        
        filterBtns.forEach(btn => {
            btn.addEventListener('click', () => {
                console.log('Milestone filter clicked:', btn.dataset.filter);
                filterBtns.forEach(b => b.classList.remove('active'));
                btn.classList.add('active');
                
                this.currentMilestoneFilter = btn.dataset.filter;
                // Re-render with cached data instead of making new API call
                if (this.milestones) {
                    this.renderMilestones(this.milestones);
                } else {
                    this.loadMilestones();
                }
            });
        });
        
        // Initialize filter
        this.currentMilestoneFilter = 'all';
    }
    
    async loadNoteLinks() {
        try {
            const response = await fetch(`${this.apiBase}/note-links`);
            const noteLinks = await response.json();
            this.renderNoteLinks(noteLinks);
        } catch (error) {
            console.error('Failed to load note links:', error);
            this.showError('Failed to load note links');
        }
    }
    
    renderNoteLinks(noteLinks) {
        const container = document.getElementById('note-link-list');
        const filteredLinks = this.filterNoteLinks(noteLinks);
        
        if (filteredLinks.length === 0) {
            container.innerHTML = '<div class="loading">No note links found</div>';
            return;
        }
        
        container.innerHTML = filteredLinks.map(link => `
            <div class="note-link-item" data-entity-id="${link.id}">
                <div class="link-header">
                    <span class="link-type-badge link-type-${link.link_type}">${this.formatLinkType(link.link_type)}</span>
                    <span class="link-direction">${link.source_note_id} → ${link.target_id}</span>
                    ${link.auto_detected ? '<span class="auto-badge">AUTO</span>' : ''}
                </div>
                <div class="link-details">
                    <span class="target-type">${link.target_type === 'entity' ? (link.target_entity_type || 'entity') : 'note'}</span>
                    ${link.detection_reason ? `<span class="detection-reason">${link.detection_reason}</span>` : ''}
                </div>
            </div>
        `).join('');
        
        // Add click listeners for note link items
        container.querySelectorAll('.note-link-item').forEach(item => {
            item.addEventListener('click', async () => {
                const linkId = item.dataset.entityId;
                try {
                    const response = await fetch(`${this.apiBase}/note-links/${linkId}`);
                    if (response.ok) {
                        const linkData = await response.json();
                        this.showEntityDialog('note-link', linkData);
                    } else {
                        console.error('Failed to fetch note link details');
                    }
                } catch (error) {
                    console.error('Error fetching note link details:', error);
                }
            });
        });
    }
    
    filterNoteLinks(noteLinks) {
        if (this.currentNoteLinkFilter === 'all') return noteLinks;
        if (this.currentNoteLinkFilter === 'auto') return noteLinks.filter(link => link.auto_detected);
        return noteLinks.filter(link => link.link_type === this.currentNoteLinkFilter);
    }
    
    formatLinkType(linkType) {
        const typeMap = {
            'reference': 'REF',
            'response_to': 'RESP',
            'related': 'REL',
            'blocks': 'BLOCKS',
            'depends_on': 'DEPS'
        };
        return typeMap[linkType] || linkType.toUpperCase();
    }
    
    renderActivity(activity) {
        const container = document.getElementById('activity-list');
        
        if (activity.length === 0) {
            container.innerHTML = '<div class="loading">No recent activity</div>';
            return;
        }
        
        container.innerHTML = activity.map(item => `
            <div class="activity-item">
                <div class="time">${this.formatTime(item.timestamp)}</div>
                <div class="description">${item.description}</div>
                <div class="entity">${item.entity_type}: ${item.entity_id}</div>
            </div>
        `).join('');
    }
    
    filterFeatures(features) {
        if (this.currentFeatureFilter === 'all') return features;
        
        return features.filter(feature => {
            switch (this.currentFeatureFilter) {
                case 'pending':
                    return feature.state === 'NotImplemented' || feature.state === 'Planned';
                case 'implemented': 
                    return feature.state === 'Implemented' || feature.state === 'InProgress';
                case 'tested':
                    return feature.test_status === 'Passed' || feature.test_status === 'Tested';
                default:
                    return true;
            }
        });
    }
    
    filterTasks(tasks) {
        if (this.currentTaskFilter === 'all') return tasks;
        
        return tasks.filter(task => {
            switch (this.currentTaskFilter) {
                case 'pending':
                    return task.status === 'Pending';
                case 'in_progress':
                    return task.status === 'InProgress';
                case 'completed':
                    return task.status === 'Completed';
                case 'blocked':
                    return task.status === 'Blocked';
                default:
                    return true;
            }
        });
    }
    
    filterDirectives(directives) {
        if (this.currentDirectiveFilter === 'all') return directives;
        
        return directives.filter(directive => {
            return directive.category.toLowerCase() === this.currentDirectiveFilter.toLowerCase();
        });
    }
    
    filterRelationships(relationships) {
        if (this.currentRelationshipFilter === 'all') return relationships;
        
        return relationships.filter(relationship => {
            return relationship.dependency_type.toLowerCase() === this.currentRelationshipFilter.toLowerCase();
        });
    }
    
    formatStatus(status) {
        const statusMap = {
            'pending': 'Pending',
            'in_progress': 'In Progress',
            'completed': 'Completed',
            'blocked': 'Blocked',
            'implemented': 'Implemented',
            'tested': 'Tested',
            'passed': 'Passed',
            'failed': 'Failed'
        };
        
        return statusMap[status] || status;
    }
    
    formatTime(timestamp) {
        const date = new Date(timestamp);
        const now = new Date();
        const diff = now - date;
        
        const minutes = Math.floor(diff / 60000);
        const hours = Math.floor(diff / 3600000);
        const days = Math.floor(diff / 86400000);
        
        if (minutes < 1) return 'Just now';
        if (minutes < 60) return `${minutes}m ago`;
        if (hours < 24) return `${hours}h ago`;
        return `${days}d ago`;
    }
    
    setupEventListeners() {
        // Feature filter buttons
        document.querySelectorAll('.feature-controls .filter-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                document.querySelectorAll('.feature-controls .filter-btn').forEach(b => 
                    b.classList.remove('active'));
                e.target.classList.add('active');
                
                this.currentFeatureFilter = e.target.dataset.filter;
                this.loadFeatures();
            });
        });
        
        // Task filter buttons
        document.querySelectorAll('.task-controls .filter-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                document.querySelectorAll('.task-controls .filter-btn').forEach(b => 
                    b.classList.remove('active'));
                e.target.classList.add('active');
                
                this.currentTaskFilter = e.target.dataset.filter;
                this.loadTasks();
            });
        });
        
        // Directive filter buttons
        document.querySelectorAll('.directive-controls .filter-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                document.querySelectorAll('.directive-controls .filter-btn').forEach(b => 
                    b.classList.remove('active'));
                e.target.classList.add('active');
                
                this.currentDirectiveFilter = e.target.dataset.filter;
                this.loadDirectives();
            });
        });
        
        // Relationship filter buttons
        document.querySelectorAll('.relationship-controls .filter-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                document.querySelectorAll('.relationship-controls .filter-btn').forEach(b => 
                    b.classList.remove('active'));
                e.target.classList.add('active');
                
                this.currentRelationshipFilter = e.target.dataset.filter;
                this.loadRelationships();
            });
        });
        
        // Note link filter buttons
        document.querySelectorAll('.note-link-controls .filter-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                document.querySelectorAll('.note-link-controls .filter-btn').forEach(b => 
                    b.classList.remove('active'));
                e.target.classList.add('active');
                
                this.currentNoteLinkFilter = e.target.dataset.filter;
                this.loadNoteLinks();
            });
        });
        
        // Milestone filter buttons
        document.querySelectorAll('.milestones .filter-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                console.log('Milestone filter clicked via main event listeners:', e.target.dataset.filter);
                document.querySelectorAll('.milestones .filter-btn').forEach(b => 
                    b.classList.remove('active'));
                e.target.classList.add('active');
                
                this.currentMilestoneFilter = e.target.dataset.filter;
                // Re-render with cached data instead of making new API call
                if (this.milestones) {
                    this.renderMilestones(this.milestones);
                } else {
                    this.loadMilestones();
                }
            });
        });
        
        // Refresh on window focus
        window.addEventListener('focus', () => {
            this.loadInitialData();
        });
    }
    
    startAutoRefresh() {
        setInterval(() => {
            this.loadInitialData();
        }, this.updateInterval);
    }
    
    updateLastUpdated() {
        const now = new Date();
        document.getElementById('last-updated').textContent = 
            now.toLocaleTimeString();
    }
    
    showError(message) {
        console.error('Dashboard error:', message);
        // Could implement toast notifications or error banners here
    }
    
    showEntityDialog(entityType, entityData) {
        // Check if modal already exists and remove it
        const existingModal = document.querySelector('.modal-backdrop');
        if (existingModal) {
            document.body.removeChild(existingModal);
        }
        
        // Create modal backdrop
        const backdrop = document.createElement('div');
        backdrop.className = 'modal-backdrop';
        
        // Create modal dialog
        const dialog = document.createElement('div');
        dialog.className = 'modal-dialog';
        
        let dialogContent = '';
        
        if (entityType === 'feature') {
            dialogContent = this.generateFeatureDialogContent(entityData);
        } else if (entityType === 'task') {
            dialogContent = this.generateTaskDialogContent(entityData);
        } else if (entityType === 'directive') {
            dialogContent = this.generateDirectiveDialogContent(entityData);
        } else if (entityType === 'milestone') {
            dialogContent = this.generateMilestoneDialogContent(entityData);
        }
        
        dialog.innerHTML = `
            <div class="modal-header">
                <h2>${entityType.charAt(0).toUpperCase() + entityType.slice(1)} Details</h2>
                <button class="modal-close" type="button">&times;</button>
            </div>
            <div class="modal-content">
                ${dialogContent}
            </div>
        `;
        
        backdrop.appendChild(dialog);
        document.body.appendChild(backdrop);
        
        // Store original overflow value before changing it
        const originalOverflow = document.body.style.overflow || '';
        document.body.style.overflow = 'hidden';
        
        // Create a single close function that properly restores scroll
        const closeModal = () => {
            // Restore original overflow
            document.body.style.overflow = originalOverflow;
            
            // Remove the modal
            if (document.body.contains(backdrop)) {
                document.body.removeChild(backdrop);
            }
            
            // Clean up event listener
            document.removeEventListener('keydown', handleEsc);
        };
        
        // Add event listeners
        const closeBtn = dialog.querySelector('.modal-close');
        closeBtn.addEventListener('click', closeModal);
        
        backdrop.addEventListener('click', (e) => {
            if (e.target === backdrop) {
                closeModal();
            }
        });
        
        // ESC key to close
        const handleEsc = (e) => {
            if (e.key === 'Escape') {
                closeModal();
            }
        };
        document.addEventListener('keydown', handleEsc);
    }
    
    generateFeatureDialogContent(feature) {
        return `
            <div class="entity-detail-section">
                <h3>Overview</h3>
                <div class="detail-grid">
                    <div class="detail-item">
                        <label>Feature Code:</label>
                        <span class="feature-code">${feature.code}</span>
                    </div>
                    <div class="detail-item">
                        <label>Name:</label>
                        <span>${feature.name}</span>
                    </div>
                    <div class="detail-item">
                        <label>Category:</label>
                        <span>${feature.category || 'Uncategorized'}</span>
                    </div>
                    <div class="detail-item">
                        <label>Priority:</label>
                        <span class="priority-badge ${(feature.priority || 'medium').toLowerCase()}">
                            ${feature.priority || 'Medium'}
                        </span>
                    </div>
                </div>
            </div>
            
            <div class="entity-detail-section">
                <h3>Description</h3>
                <div class="description-content">
                    ${feature.description}
                </div>
            </div>
            
            <div class="entity-detail-section">
                <h3>Status</h3>
                <div class="detail-grid">
                    <div class="detail-item">
                        <label>Implementation Status:</label>
                        <span class="status-badge ${feature.state.toLowerCase()}">
                            ${this.formatFeatureState(feature.state)}
                        </span>
                    </div>
                    <div class="detail-item">
                        <label>Test Status:</label>
                        <span class="status-badge ${feature.test_status.toLowerCase()}">
                            ${this.formatFeatureState(feature.test_status)}
                        </span>
                    </div>
                </div>
            </div>
            
            ${feature.notes ? `
            <div class="entity-detail-section">
                <h3>Notes</h3>
                <div class="notes-content">
                    ${feature.notes}
                </div>
            </div>
            ` : ''}
            
            <div class="entity-detail-section">
                <h3>Timestamps</h3>
                <div class="detail-grid">
                    <div class="detail-item">
                        <label>Created:</label>
                        <span>${this.formatDateTime(feature.created_at)}</span>
                    </div>
                    <div class="detail-item">
                        <label>Updated:</label>
                        <span>${this.formatDateTime(feature.updated_at)}</span>
                    </div>
                </div>
            </div>
        `;
    }
    
    generateTaskDialogContent(task) {
        return `
            <div class="entity-detail-section">
                <h3>Overview</h3>
                <div class="detail-grid">
                    <div class="detail-item">
                        <label>Task ID:</label>
                        <span class="task-id">${task.id}</span>
                    </div>
                    <div class="detail-item">
                        <label>Title:</label>
                        <span>${task.title}</span>
                    </div>
                    <div class="detail-item">
                        <label>Category:</label>
                        <span>${task.category || 'General'}</span>
                    </div>
                    <div class="detail-item">
                        <label>Priority:</label>
                        <span class="priority-badge ${(task.priority || 'medium').toLowerCase()}">
                            ${task.priority || 'Medium'}
                        </span>
                    </div>
                </div>
            </div>
            
            <div class="entity-detail-section">
                <h3>Description</h3>
                <div class="description-content">
                    ${task.description}
                </div>
            </div>
            
            <div class="entity-detail-section">
                <h3>Status</h3>
                <div class="detail-grid">
                    <div class="detail-item">
                        <label>Current Status:</label>
                        <span class="status-badge ${task.status.toLowerCase()}">
                            ${this.formatStatus(task.status)}
                        </span>
                    </div>
                    ${task.feature_id ? `
                    <div class="detail-item">
                        <label>Related Feature:</label>
                        <span class="feature-link">${task.feature_id}</span>
                    </div>
                    ` : ''}
                </div>
            </div>
            
            ${task.notes ? `
            <div class="entity-detail-section">
                <h3>Notes</h3>
                <div class="notes-content">
                    ${task.notes}
                </div>
            </div>
            ` : ''}
            
            <div class="entity-detail-section">
                <h3>Timestamps</h3>
                <div class="detail-grid">
                    <div class="detail-item">
                        <label>Created:</label>
                        <span>${this.formatDateTime(task.created_at)}</span>
                    </div>
                    <div class="detail-item">
                        <label>Updated:</label>
                        <span>${this.formatDateTime(task.updated_at)}</span>
                    </div>
                </div>
            </div>
        `;
    }
    
    generateDirectiveDialogContent(directive) {
        return `
            <div class="entity-detail-section">
                <h3>Overview</h3>
                <div class="detail-grid">
                    <div class="detail-item">
                        <label>Directive Code:</label>
                        <span class="directive-code">${directive.code}</span>
                    </div>
                    <div class="detail-item">
                        <label>Title:</label>
                        <span>${directive.title}</span>
                    </div>
                    <div class="detail-item">
                        <label>Category:</label>
                        <span class="category-badge ${directive.category.toLowerCase()}">
                            ${directive.category}
                        </span>
                    </div>
                    <div class="detail-item">
                        <label>Priority:</label>
                        <span class="priority-badge ${directive.priority.toLowerCase()}">
                            ${directive.priority}
                        </span>
                    </div>
                </div>
            </div>
            
            <div class="entity-detail-section">
                <h3>Rule</h3>
                <div class="description-content">
                    ${directive.rule}
                </div>
            </div>
            
            <div class="entity-detail-section">
                <h3>Status</h3>
                <div class="detail-grid">
                    <div class="detail-item">
                        <label>Active:</label>
                        <span class="status-badge ${directive.active ? 'active' : 'inactive'}">
                            ${directive.active ? 'Active' : 'Inactive'}
                        </span>
                    </div>
                    ${directive.compliance_checked ? `
                    <div class="detail-item">
                        <label>Last Checked:</label>
                        <span>${this.formatDateTime(directive.compliance_checked)}</span>
                    </div>
                    ` : ''}
                </div>
            </div>
            
            ${directive.context ? `
            <div class="entity-detail-section">
                <h3>Context</h3>
                <div class="description-content">
                    ${directive.context}
                </div>
            </div>
            ` : ''}
            
            ${directive.rationale ? `
            <div class="entity-detail-section">
                <h3>Rationale</h3>
                <div class="description-content">
                    ${directive.rationale}
                </div>
            </div>
            ` : ''}
            
            ${directive.examples ? `
            <div class="entity-detail-section">
                <h3>Examples</h3>
                <div class="description-content">
                    ${directive.examples}
                </div>
            </div>
            ` : ''}
            
            ${directive.violations ? `
            <div class="entity-detail-section">
                <h3>Violation Consequences</h3>
                <div class="description-content">
                    ${directive.violations}
                </div>
            </div>
            ` : ''}
            
            <div class="entity-detail-section">
                <h3>Timestamps</h3>
                <div class="detail-grid">
                    <div class="detail-item">
                        <label>Created:</label>
                        <span>${this.formatDateTime(directive.created_at)}</span>
                    </div>
                    <div class="detail-item">
                        <label>Updated:</label>
                        <span>${this.formatDateTime(directive.updated_at)}</span>
                    </div>
                    ${directive.archived_at ? `
                    <div class="detail-item">
                        <label>Archived:</label>
                        <span>${this.formatDateTime(directive.archived_at)}</span>
                    </div>
                    ` : ''}
                </div>
            </div>
        `;
    }
    
    generateMilestoneDialogContent(milestone) {
        return `
            <div class="entity-detail-section">
                <h3>Overview</h3>
                <div class="detail-grid">
                    <div class="detail-item">
                        <label>Milestone ID:</label>
                        <span class="milestone-id">${milestone.id}</span>
                    </div>
                    <div class="detail-item">
                        <label>Title:</label>
                        <span>${milestone.title}</span>
                    </div>
                    <div class="detail-item">
                        <label>Status:</label>
                        <span class="status-badge ${milestone.status.toLowerCase()}">
                            ${this.formatMilestoneStatus(milestone.status)}
                        </span>
                    </div>
                    <div class="detail-item">
                        <label>Completion:</label>
                        <div class="completion-display">
                            <div class="completion-bar-large">
                                <div class="completion-fill" style="width: ${milestone.completion_percentage}%"></div>
                            </div>
                            <span class="completion-text">${Math.round(milestone.completion_percentage)}%</span>
                        </div>
                    </div>
                </div>
            </div>
            
            <div class="entity-detail-section">
                <h3>Description</h3>
                <div class="description-content">
                    ${milestone.description}
                </div>
            </div>
            
            ${milestone.target_date || milestone.achieved_date ? `
            <div class="entity-detail-section">
                <h3>Dates</h3>
                <div class="detail-grid">
                    ${milestone.target_date ? `
                    <div class="detail-item">
                        <label>Target Date:</label>
                        <span>${this.formatDateTime(milestone.target_date)}</span>
                    </div>
                    ` : ''}
                    ${milestone.achieved_date ? `
                    <div class="detail-item">
                        <label>Achieved Date:</label>
                        <span>${this.formatDateTime(milestone.achieved_date)}</span>
                    </div>
                    ` : ''}
                </div>
            </div>
            ` : ''}
            
            ${milestone.feature_ids ? `
            <div class="entity-detail-section">
                <h3>Linked Features</h3>
                <div class="feature-links">
                    ${JSON.parse(milestone.feature_ids).map(featureId => 
                        `<span class="feature-link">${featureId}</span>`
                    ).join('')}
                </div>
            </div>
            ` : ''}
            
            ${milestone.success_criteria ? `
            <div class="entity-detail-section">
                <h3>Success Criteria</h3>
                <ul class="criteria-list">
                    ${JSON.parse(milestone.success_criteria).map(criterion => 
                        `<li>${criterion}</li>`
                    ).join('')}
                </ul>
            </div>
            ` : ''}
            
            <div class="entity-detail-section">
                <h3>Timestamps</h3>
                <div class="detail-grid">
                    <div class="detail-item">
                        <label>Created:</label>
                        <span>${this.formatDateTime(milestone.created_at)}</span>
                    </div>
                    <div class="detail-item">
                        <label>Updated:</label>
                        <span>${this.formatDateTime(milestone.updated_at)}</span>
                    </div>
                </div>
            </div>
        `;
    }
    
    formatFeatureState(state) {
        const stateMap = {
            'NotImplemented': '❌ Not Implemented',
            'InProgress': '🟠 In Progress', 
            'Implemented': '🟢 Implemented',
            'Tested': '🟢 Tested',
            'Failed': '🔴 Failed',
            'Warning': '⚠️ Warning'
        };
        
        return stateMap[state] || state;
    }
    
    formatDateTime(timestamp) {
        if (!timestamp) return 'Unknown';
        
        const date = new Date(timestamp);
        return date.toLocaleString();
    }

    // Git Integration Methods
    
    async initializeGitIntegration() {
        try {
            // Initialize Monaco Editor
            await this.initializeMonaco();
            
            // Load git data
            await Promise.all([
                this.loadTimeline(),
                this.loadCommits()
            ]);
            
            // Setup git-related event listeners
            this.setupGitEventListeners();
            
        } catch (error) {
            console.error('Failed to initialize git integration:', error);
        }
    }
    
    async initializeMonaco() {
        return new Promise((resolve, reject) => {
            require.config({ 
                paths: { 
                    'vs': 'https://unpkg.com/monaco-editor@0.44.0/min/vs' 
                } 
            });
            
            require(['vs/editor/editor.main'], () => {
                try {
                    // Initialize main file editor
                    this.monacoEditor = monaco.editor.create(
                        document.getElementById('monaco-container'), 
                        {
                            value: '// Select a file to view its contents',
                            language: 'plaintext',
                            theme: 'vs',
                            readOnly: true,
                            minimap: { enabled: false },
                            scrollBeyondLastLine: false,
                            automaticLayout: true
                        }
                    );
                    
                    resolve();
                } catch (error) {
                    reject(error);
                }
            });
        });
    }
    
    setupGitEventListeners() {
        // Tab switching
        document.getElementById('timeline-btn').addEventListener('click', () => {
            this.switchView('timeline');
        });
        
        document.getElementById('files-btn').addEventListener('click', () => {
            this.switchView('files');
        });
        
        document.getElementById('commits-btn').addEventListener('click', () => {
            this.switchView('commits');
        });
        
        // Commit selector
        document.getElementById('commit-select').addEventListener('change', (e) => {
            this.selectedCommit = e.target.value;
            this.loadFilesForCommit(this.selectedCommit);
        });
        
        // Show diff button
        document.getElementById('show-diff-btn').addEventListener('click', () => {
            if (this.selectedFile) {
                this.showFileDiff(this.selectedFile);
            }
        });
        
        // Modal close
        const modal = document.getElementById('diff-modal');
        const closeBtn = modal.querySelector('.modal-close');
        
        closeBtn.addEventListener('click', () => {
            modal.style.display = 'none';
            if (this.diffEditor) {
                this.diffEditor.dispose();
                this.diffEditor = null;
            }
        });
        
        // Close modal on background click
        modal.addEventListener('click', (e) => {
            if (e.target === modal) {
                modal.style.display = 'none';
                if (this.diffEditor) {
                    this.diffEditor.dispose();
                    this.diffEditor = null;
                }
            }
        });
    }
    
    switchView(view) {
        // Update active tab
        document.querySelectorAll('.tab-btn').forEach(btn => btn.classList.remove('active'));
        document.getElementById(`${view}-btn`).classList.add('active');
        
        // Show/hide content
        document.querySelectorAll('.timeline-content, .files-content, .commits-content').forEach(content => {
            content.style.display = 'none';
        });
        document.getElementById(`${view}-view`).style.display = 'block';
        
        this.currentView = view;
        
        // Load data for current view if needed
        if (view === 'files' && this.files.length === 0) {
            this.loadFilesForCommit(this.selectedCommit);
        }
    }
    
    async loadTimeline() {
        try {
            const response = await fetch(`${this.apiBase}/git/timeline`);
            if (!response.ok) throw new Error('Failed to fetch timeline');
            
            const data = await response.json();
            this.renderTimeline(data.items);
        } catch (error) {
            console.error('Error loading timeline:', error);
            document.getElementById('timeline-list').innerHTML = 
                '<div class="error">Failed to load timeline</div>';
        }
    }
    
    async loadCommits() {
        try {
            const response = await fetch(`${this.apiBase}/git/commits?limit=50`);
            if (!response.ok) throw new Error('Failed to fetch commits');
            
            this.commits = await response.json();
            this.renderCommits(this.commits);
            this.populateCommitSelector(this.commits);
        } catch (error) {
            console.error('Error loading commits:', error);
            document.getElementById('commits-list').innerHTML = 
                '<div class="error">Failed to load commits</div>';
        }
    }
    
    async loadFilesForCommit(commitHash) {
        try {
            // Get list of files at this commit
            const response = await fetch(`${this.apiBase}/git/commits/${commitHash}`);
            if (!response.ok) throw new Error('Failed to fetch commit details');
            
            const commit = await response.json();
            this.files = commit.files_changed || [];
            this.renderFileTree(this.files);
        } catch (error) {
            console.error('Error loading files:', error);
            document.getElementById('file-tree').innerHTML = 
                '<div class="error">Failed to load files</div>';
        }
    }
    
    renderTimeline(items) {
        const container = document.getElementById('timeline-list');
        
        if (!items || items.length === 0) {
            container.innerHTML = '<div class="no-data">No timeline items found</div>';
            return;
        }
        
        container.innerHTML = items.map(item => `
            <div class="timeline-item">
                <div class="timeline-icon ${item.item_type}">
                    ${item.item_type === 'session' ? '⚡' : '📝'}
                </div>
                <div class="timeline-content-item">
                    <div class="timeline-title">${item.title}</div>
                    ${item.description ? `<div class="timeline-description">${item.description}</div>` : ''}
                    <div class="timeline-meta">
                        <span class="timeline-time">${this.formatDateTime(item.timestamp)}</span>
                        ${item.commit_hash ? `<span class="commit-hash">${item.commit_hash.substring(0, 8)}</span>` : ''}
                        ${item.session_id ? `<span>Session: ${item.session_id}</span>` : ''}
                    </div>
                </div>
            </div>
        `).join('');
    }
    
    renderCommits(commits) {
        const container = document.getElementById('commits-list');
        
        if (!commits || commits.length === 0) {
            container.innerHTML = '<div class="no-data">No commits found</div>';
            return;
        }
        
        container.innerHTML = commits.map(commit => `
            <div class="commit-item" data-hash="${commit.hash}">
                <div class="commit-header">
                    <div>
                        <div class="commit-message">${commit.message}</div>
                        <div class="commit-author">by ${commit.author}</div>
                    </div>
                    <div class="commit-hash-display">${commit.hash.substring(0, 8)}</div>
                </div>
                <div class="commit-stats">
                    <span>📅 ${this.formatDateTime(commit.date)}</span>
                    <span>➕ ${commit.insertions} insertions</span>
                    <span>➖ ${commit.deletions} deletions</span>
                    <span>📄 ${commit.files_changed.length} files</span>
                </div>
                ${commit.files_changed.length > 0 ? `
                <div class="commit-files">
                    <div class="commit-files-title">Files changed:</div>
                    <div class="commit-files-list">
                        ${commit.files_changed.slice(0, 5).map(file => 
                            `<span class="commit-file">${file}</span>`
                        ).join('')}
                        ${commit.files_changed.length > 5 ? 
                            `<span class="commit-file">+${commit.files_changed.length - 5} more</span>` : ''
                        }
                    </div>
                </div>
                ` : ''}
            </div>
        `).join('');
    }
    
    renderFileTree(files) {
        const container = document.getElementById('file-tree');
        
        if (!files || files.length === 0) {
            container.innerHTML = '<div class="no-data">No files found</div>';
            return;
        }
        
        container.innerHTML = files.map(file => `
            <div class="file-item" data-path="${file}" ${file === this.selectedFile ? 'class="active"' : ''}>
                <span class="file-icon">📄</span>
                <span class="file-name">${file}</span>
            </div>
        `).join('');
        
        // Add click handlers for file items
        container.querySelectorAll('.file-item').forEach(item => {
            item.addEventListener('click', () => {
                this.selectFile(item.dataset.path);
            });
        });
    }
    
    populateCommitSelector(commits) {
        const selector = document.getElementById('commit-select');
        const currentOptions = Array.from(selector.options).map(opt => opt.value);
        
        // Add new commits to selector
        commits.forEach(commit => {
            if (!currentOptions.includes(commit.hash)) {
                const option = document.createElement('option');
                option.value = commit.hash;
                option.textContent = `${commit.hash.substring(0, 8)} - ${commit.message.substring(0, 50)}${commit.message.length > 50 ? '...' : ''}`;
                selector.appendChild(option);
            }
        });
    }
    
    async selectFile(filePath) {
        this.selectedFile = filePath;
        
        // Update file tree selection
        document.querySelectorAll('.file-item').forEach(item => {
            item.classList.toggle('active', item.dataset.path === filePath);
        });
        
        // Update file header
        document.getElementById('current-file-path').textContent = filePath;
        document.getElementById('show-diff-btn').disabled = false;
        
        // Load file content
        try {
            const response = await fetch(`${this.apiBase}/git/files/${this.selectedCommit}/${encodeURIComponent(filePath)}`);
            if (!response.ok) throw new Error('Failed to fetch file content');
            
            const data = await response.json();
            const language = this.detectLanguage(filePath);
            
            if (this.monacoEditor) {
                monaco.editor.setModelLanguage(this.monacoEditor.getModel(), language);
                this.monacoEditor.setValue(data.content);
            }
        } catch (error) {
            console.error('Error loading file content:', error);
            if (this.monacoEditor) {
                this.monacoEditor.setValue(`// Error loading file: ${error.message}`);
            }
        }
    }
    
    async showFileDiff(filePath) {
        const modal = document.getElementById('diff-modal');
        const container = document.getElementById('diff-container');
        
        try {
            // Get current and previous version of the file
            const currentCommit = this.selectedCommit;
            const previousCommit = `${currentCommit}~1`;
            
            const [currentResponse, previousResponse] = await Promise.all([
                fetch(`${this.apiBase}/git/files/${currentCommit}/${encodeURIComponent(filePath)}`),
                fetch(`${this.apiBase}/git/files/${previousCommit}/${encodeURIComponent(filePath)}`)
            ]);
            
            const currentData = currentResponse.ok ? await currentResponse.json() : { content: '' };
            const previousData = previousResponse.ok ? await previousResponse.json() : { content: '' };
            
            // Create diff editor
            this.diffEditor = monaco.editor.createDiffEditor(container, {
                theme: 'vs',
                readOnly: true,
                minimap: { enabled: false },
                automaticLayout: true
            });
            
            const language = this.detectLanguage(filePath);
            
            const originalModel = monaco.editor.createModel(previousData.content || '', language);
            const modifiedModel = monaco.editor.createModel(currentData.content || '', language);
            
            this.diffEditor.setModel({
                original: originalModel,
                modified: modifiedModel
            });
            
            modal.style.display = 'flex';
            
        } catch (error) {
            console.error('Error showing diff:', error);
            container.innerHTML = `<div class="error">Failed to load diff: ${error.message}</div>`;
            modal.style.display = 'flex';
        }
    }
    
    detectLanguage(filePath) {
        const ext = filePath.split('.').pop().toLowerCase();
        const langMap = {
            'rs': 'rust',
            'js': 'javascript',
            'ts': 'typescript',
            'html': 'html',
            'css': 'css',
            'json': 'json',
            'md': 'markdown',
            'py': 'python',
            'java': 'java',
            'cpp': 'cpp',
            'c': 'c',
            'sh': 'shell',
            'yml': 'yaml',
            'yaml': 'yaml',
            'toml': 'toml',
            'xml': 'xml',
            'sql': 'sql'
        };
        
        return langMap[ext] || 'plaintext';
    }
}

// Initialize dashboard when page loads
document.addEventListener('DOMContentLoaded', () => {
    new Dashboard();
});