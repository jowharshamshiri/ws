// Workspace Project Dashboard JavaScript

class Dashboard {
    constructor() {
        this.apiBase = '/api';
        this.updateInterval = 5000; // 5 seconds
        this.currentFeatureFilter = 'all';
        this.currentTaskFilter = 'all';
        
        this.init();
    }
    
    async init() {
        await this.loadInitialData();
        this.setupEventListeners();
        this.startAutoRefresh();
        
        console.log('Dashboard initialized');
    }
    
    async loadInitialData() {
        try {
            await Promise.all([
                this.loadProjectStatus(),
                this.loadFeatures(),
                this.loadTasks(),
                this.loadActivity()
            ]);
            
            this.updateLastUpdated();
        } catch (error) {
            console.error('Failed to load initial data:', error);
            this.showError('Failed to load dashboard data');
        }
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
            <div class="feature-item" data-status="${feature.state}">
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
    }
    
    renderTasks(tasks) {
        const container = document.getElementById('task-list');
        
        if (tasks.length === 0) {
            container.innerHTML = '<div class="loading">No tasks found</div>';
            return;
        }
        
        const filteredTasks = this.filterTasks(tasks);
        
        container.innerHTML = filteredTasks.map(task => `
            <div class="task-item" data-status="${task.status}">
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
}

// Initialize dashboard when page loads
document.addEventListener('DOMContentLoaded', () => {
    new Dashboard();
});