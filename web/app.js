
let currentSessionId = null;
let messages = [];
let ws = null;
let wsConnected = false;

const API_BASE = window.location.origin;

const DOM = {
    chatContainer: document.getElementById('chat-container'),
    welcomeScreen: document.getElementById('welcome-screen'),
    chatInput: document.getElementById('chat-input'),
    sendBtn: document.getElementById('send-btn'),
    newChatBtn: document.getElementById('new-chat-btn'),
    sessionsList: document.getElementById('sessions-list'),
    connectionStatus: document.getElementById('connection-status'),
};

function init() {
    setupEventListeners();
    connectWebSocket();
    loadSessions();
}

function updateConnectionStatus(connected) {
    if (connected) {
        DOM.connectionStatus.textContent = 'Connected';
        DOM.connectionStatus.className = 'connection-status connected';
    } else {
        DOM.connectionStatus.textContent = 'Disconnected';
        DOM.connectionStatus.className = 'connection-status disconnected';
    }
}

function setupEventListeners() {
    DOM.newChatBtn.addEventListener('click', createNewSession);
    DOM.sendBtn.addEventListener('click', sendMessage);

    DOM.chatInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            sendMessage();
        }
    });

    document.querySelectorAll('.quick-action').forEach(btn => {
        btn.addEventListener('click', () => {
            const prompt = btn.dataset.prompt;
            if (!currentSessionId) {
                createNewSession().then(() => {
                    DOM.chatInput.value = prompt;
                    sendMessage();
                });
            } else {
                DOM.chatInput.value = prompt;
                sendMessage();
            }
        });
    });

    DOM.chatInput.addEventListener('input', () => {
        DOM.chatInput.style.height = 'auto';
        DOM.chatInput.style.height = DOM.chatInput.scrollHeight + 'px';
    });
}

function connectWebSocket() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/ws`;

    ws = new WebSocket(wsUrl);

    ws.onopen = () => {
        console.log('WebSocket connected');
        wsConnected = true;
        updateConnectionStatus(true);
    };

    ws.onmessage = (event) => {
        try {
            const msg = JSON.parse(event.data);
            handleWsMessage(msg);
        } catch (e) {
            console.error('Failed to parse WebSocket message:', e);
        }
    };

    ws.onclose = () => {
        console.log('WebSocket disconnected');
        wsConnected = false;
        updateConnectionStatus(false);
        setTimeout(connectWebSocket, 3000);
    };

    ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        updateConnectionStatus(false);
    };
}

function handleWsMessage(msg) {
    console.log('Received message:', msg);
    switch (msg.msg_type) {
        case 'session_created':
            currentSessionId = msg.content;
            DOM.welcomeScreen.style.display = 'none';
            loadSessions();
            break;

        case 'thinking':
            showTypingIndicator();
            break;

        case 'response':
            removeTypingIndicator();
            messages.push({ role: 'assistant', content: msg.content });
            renderMessages();
            break;

        case 'error':
            removeTypingIndicator();
            messages.push({ role: 'assistant', content: `Error: ${msg.content}` });
            renderMessages();
            break;

        default:
            console.log('Unknown message type:', msg.msg_type);
    }
}

async function createNewSession() {
    if (!wsConnected) {
        console.error('WebSocket not connected');
        alert('WebSocket connection not available. Please wait...');
        return;
    }

    const promise = new Promise((resolve) => {
        const check = setInterval(() => {
            if (currentSessionId) {
                clearInterval(check);
                resolve();
            }
        }, 100);
    });

    ws.send(JSON.stringify({ msg_type: 'create_session', content: '' }));
    messages = [];
    
    return promise;
}

async function loadSessions() {
    try {
        const response = await fetch(`${API_BASE}/sessions`);
        if (response.ok) {
            const sessions = await response.json();
            renderSessions(sessions);
        }
    } catch (error) {
        console.error('Failed to load sessions:', error);
    }
}

function renderSessions(sessions) {
    DOM.sessionsList.innerHTML = '';

    if (sessions.length === 0) {
        const empty = document.createElement('div');
        empty.className = 'session-empty';
        empty.textContent = 'No sessions';
        empty.style.cssText = 'padding: 12px; color: #94a3b8; font-size: 13px;';
        DOM.sessionsList.appendChild(empty);
        return;
    }

    sessions.forEach(sessionId => {
        const item = document.createElement('div');
        item.className = `session-item ${sessionId === currentSessionId ? 'active' : ''}`;
        item.textContent = sessionId.substring(0, 8) + '...';
        item.addEventListener('click', () => {
            currentSessionId = sessionId;
            messages = [];
            DOM.welcomeScreen.style.display = 'none';
            renderSessions(sessions);
        });
        DOM.sessionsList.appendChild(item);
    });
}

async function sendMessage() {
    const input = DOM.chatInput.value.trim();
    if (!input) return;

    if (!currentSessionId) {
        await createNewSession();
    }

    DOM.chatInput.value = '';
    DOM.chatInput.style.height = 'auto';
    DOM.sendBtn.disabled = true;

    messages.push({ role: 'user', content: input });
    renderMessages();

    if (wsConnected && ws) {
        ws.send(JSON.stringify({
            msg_type: 'chat',
            content: input
        }));
    } else {
        alert('WebSocket connection not available');
        DOM.sendBtn.disabled = false;
    }
}

function renderMessages() {
    if (messages.length === 0) {
        DOM.welcomeScreen.style.display = 'block';
        return;
    }

    DOM.welcomeScreen.style.display = 'none';

    const existingMessages = DOM.chatContainer.querySelectorAll('.message');
    existingMessages.forEach(el => el.remove());

    messages.forEach(msg => {
        const messageEl = document.createElement('div');
        messageEl.className = `message ${msg.role}`;

        const avatar = msg.role === 'user' ? 'U' : 'A';
        const name = msg.role === 'user' ? 'You' : 'Hermes';

        messageEl.innerHTML = `
            <div class="message-header">
                <div class="message-avatar">${avatar}</div>
                <div class="message-role">${name}</div>
            </div>
            <div class="message-content">${escapeHtml(msg.content)}</div>
        `;

        DOM.chatContainer.appendChild(messageEl);
    });

    DOM.chatContainer.scrollTop = DOM.chatContainer.scrollHeight;
    DOM.sendBtn.disabled = false;
}

function showTypingIndicator() {
    const typing = document.createElement('div');
    typing.className = 'message assistant typing';
    typing.id = 'typing-indicator';
    typing.innerHTML = `
        <div class="message-header">
            <div class="message-avatar">A</div>
            <div class="message-role">Hermes</div>
        </div>
        <div class="message-content">
            <span class="typing-dots">Thinking<span>.</span><span>.</span><span>.</span></span>
        </div>
    `;
    DOM.chatContainer.appendChild(typing);
    DOM.chatContainer.scrollTop = DOM.chatContainer.scrollHeight;
}

function removeTypingIndicator() {
    const indicator = document.getElementById('typing-indicator');
    if (indicator) {
        indicator.remove();
    }
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

async function healthCheck() {
    try {
        const response = await fetch(`${API_BASE}/health`);
        if (response.ok) {
            console.log('Server is running');
        }
    } catch (error) {
        console.warn('Server not available:', error);
    }
}

healthCheck();
init();
