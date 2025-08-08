const API_BASE = '';

class ApiService {
  async request(endpoint, options = {}) {
    const url = `${API_BASE}/api${endpoint}`;
    
    const defaultOptions = {
      headers: {
        'Content-Type': 'application/json',
      },
    };

    const response = await fetch(url, { ...defaultOptions, ...options });
    
    if (!response.ok) {
      throw new Error(`API request failed: ${response.statusText}`);
    }
    
    return response.json();
  }

  // Project methods
  async getProject() {
    return this.request('/project');
  }

  // Feature methods
  async getFeatures() {
    return this.request('/features');
  }

  async updateFeature(id, updates) {
    return this.request(`/features/${id}`, {
      method: 'PUT',
      body: JSON.stringify(updates)
    });
  }

  async addFeature(feature) {
    return this.request('/features', {
      method: 'POST',
      body: JSON.stringify(feature)
    });
  }

  // Task methods
  async getTasks() {
    return this.request('/tasks');
  }

  async updateTask(id, updates) {
    return this.request(`/tasks/${id}`, {
      method: 'PUT',
      body: JSON.stringify(updates)
    });
  }

  async addTask(task) {
    return this.request('/tasks', {
      method: 'POST',
      body: JSON.stringify(task)
    });
  }

  // Session methods
  async getSessions() {
    return this.request('/sessions');
  }

  async getSession(id) {
    return this.request(`/sessions/${id}`);
  }

  // Milestone methods
  async getMilestones() {
    return this.request('/milestones');
  }

  async updateMilestone(id, updates) {
    return this.request(`/milestones/${id}`, {
      method: 'PUT',
      body: JSON.stringify(updates)
    });
  }

  // Note methods
  async getNotes() {
    return this.request('/notes');
  }

  async addNote(note) {
    return this.request('/notes', {
      method: 'POST',
      body: JSON.stringify(note)
    });
  }

  // Directive methods
  async getDirectives() {
    return this.request('/directives');
  }

  // Statistics and metrics
  async getMetrics() {
    return this.request('/metrics');
  }

  async getProjectStats() {
    return this.request('/project/stats');
  }
}

export const apiService = new ApiService();