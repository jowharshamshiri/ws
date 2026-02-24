class WebSocketService {
  constructor() {
    this.wsb = null;
    this.reconnectAttempts = 0;
    this.maxReconnectAttempts = 10;
    this.reconnectDelay = 1000;
    this.messageHandlers = [];
    this.connectionStore = null;
  }

  setConnectionStore(store) {
    this.connectionStore = store;
  }

  connect() {
    try {
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'wsb:';
      const wsUrl = `${protocol}//${window.location.host}/wsb`;
      
      this.wsb = new WebSocket(wsUrl);
      
      this.wsb.onopen = () => {
        console.log('WebSocket connected');
        if (this.connectionStore) {
          this.connectionStore.set(true);
        }
        this.reconnectAttempts = 0;
      };
      
      this.wsb.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          this.messageHandlers.forEach(handler => handler(data));
        } catch (error) {
          console.error('Failed to parse WebSocket message:', error);
        }
      };
      
      this.wsb.onclose = () => {
        console.log('WebSocket disconnected');
        if (this.connectionStore) {
          this.connectionStore.set(false);
        }
        this.scheduleReconnect();
      };
      
      this.wsb.onerror = (error) => {
        console.error('WebSocket error:', error);
        if (this.connectionStore) {
          this.connectionStore.set(false);
        }
      };
      
    } catch (error) {
      console.error('Failed to connect WebSocket:', error);
      this.scheduleReconnect();
    }
  }

  scheduleReconnect() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      setTimeout(() => {
        console.log(`Attempting to reconnect WebSocket (${this.reconnectAttempts + 1}/${this.maxReconnectAttempts})`);
        this.reconnectAttempts++;
        this.connect();
      }, this.reconnectDelay * Math.pow(2, this.reconnectAttempts));
    } else {
      console.log('Max reconnection attempts reached');
    }
  }

  onMessage(handler) {
    this.messageHandlers.push(handler);
  }

  send(data) {
    if (this.wsb && this.wsb.readyState === WebSocket.OPEN) {
      this.wsb.send(JSON.stringify(data));
    } else {
      console.warn('WebSocket not connected, cannot send message');
    }
  }

  disconnect() {
    if (this.wsb) {
      this.wsb.close();
      this.wsb = null;
    }
  }
}

export { WebSocketService };
export const websocketService = new WebSocketService();