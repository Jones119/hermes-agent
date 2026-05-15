
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
    settingsBtn: document.getElementById('settings-btn'),
    settingsModal: document.getElementById('settings-modal'),
    modalCloseBtn: document.getElementById('modal-close-btn'),
    configContent: document.getElementById('config-content'),
    toolsContent: document.getElementById('tools-content'),
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
    DOM.settingsBtn.addEventListener('click', openSettings);
    DOM.modalCloseBtn.addEventListener('click', closeSettings);

    DOM.settingsModal.addEventListener('click', (e) => {
        if (e.target === DOM.settingsModal) {
            closeSettings();
        }
    });

    document.querySelectorAll('.modal-tab').forEach(tab => {
        tab.addEventListener('click', () => {
            switchTab(tab.dataset.tab);
        });
    });

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

function switchTab(tabName) {
    document.querySelectorAll('.modal-tab').forEach(tab => {
        tab.classList.remove('active');
    });
    document.querySelectorAll('.tab-content').forEach(content => {
        content.classList.remove('active');
    });

    document.querySelector(`[data-tab="${tabName}"]`).classList.add('active');
    document.getElementById(`tab-${tabName}`).classList.add('active');

    if (tabName === 'config') {
        loadConfig();
    } else if (tabName === 'tools') {
        loadTools();
    }
}

function openSettings() {
    DOM.settingsModal.classList.add('show');
    loadConfig();
}

function closeSettings() {
    DOM.settingsModal.classList.remove('show');
}

async function loadConfig() {
    try {
        DOM.configContent.innerHTML = '<div class="loading">Loading configuration...</div>';
        const response = await fetch(`${API_BASE}/config`);
        if (response.ok) {
            const config = await response.json();
            renderConfig(config);
        } else {
            DOM.configContent.innerHTML = '<div class="empty-state">Failed to load configuration</div>';
        }
    } catch (error) {
        DOM.configContent.innerHTML = '<div class="empty-state">Failed to load configuration</div>';
        console.error('Failed to load config:', error);
    }
}

function renderConfig(config) {
    let html = '';

    if (config.llm) {
        html += `
            <div class="config-section">
                <div class="config-section-title">LLM Settings</div>
                <div class="config-item"><div class="config-label">Provider</div><div class="config-value">${config.llm.provider || 'N/A'}</div></div>
                <div class="config-item"><div class="config-label">Model</div><div class="config-value">${config.llm.model || 'N/A'}</div></div>
                <div class="config-item"><div class="config-label">Temperature</div><div class="config-value">${config.llm.temperature || '0.7'}</div></div>
                <div class="config-item"><div class="config-label">Max Tokens</div><div class="config-value">${config.llm.max_tokens || '4096'}</div></div>
                <div class="config-item"><div class="config-label">Base URL</div><div class="config-value">${config.llm.base_url || '(default)'}</div></div>
            </div>
        `;
    }

    if (config.agent) {
        html += `
            <div class="config-section">
                <div class="config-section-title">Agent Settings</div>
                <div class="config-item"><div class="config-label">Max Iterations</div><div class="config-value">${config.agent.max_iterations || '20'}</div></div>
                <div class="config-item"><div class="config-label">Max Tool Calls</div><div class="config-value">${config.agent.max_tool_calls || '50'}</div></div>
                <div class="config-item"><div class="config-label">Timeout (secs)</div><div class="config-value">${config.agent.timeout_secs || '600'}</div></div>
                <div class="config-item"><div class="config-label">Memory Limit</div><div class="config-value">${config.agent.memory_limit || '100'}</div></div>
            </div>
        `;
    }

    if (config.web) {
        html += `
            <div class="config-section">
                <div class="config-section-title">Web Server</div>
                <div class="config-item"><div class="config-label">Host</div><div class="config-value">${config.web.host || '127.0.0.1'}</div></div>
                <div class="config-item"><div class="config-label">Port</div><div class="config-value">${config.web.port || '3000'}</div></div>
                <div class="config-item"><div class="config-label">CORS Enabled</div><div class="config-value">${config.web.enable_cors ? 'Yes' : 'No'}</div></div>
            </div>
        `;
    }

    DOM.configContent.innerHTML = html || '<div class="empty-state">No configuration available</div>';
}

async function loadTools() {
    try {
        DOM.toolsContent.innerHTML = '<div class="loading">Loading tools...</div>';
        const response = await fetch(`${API_BASE}/tools`);
        if (response.ok) {
            const tools = await response.json();
            renderTools(tools);
        } else {
            DOM.toolsContent.innerHTML = '<div class="empty-state">Failed to load tools</div>';
        }
    } catch (error) {
        DOM.toolsContent.innerHTML = '<div class="empty-state">Failed to load tools</div>';
        console.error('Failed to load tools:', error);
    }
}

function renderTools(tools) {
    if (!tools || tools.length === 0) {
        DOM.toolsContent.innerHTML = '<div class="empty-state">No tools available</div>';
        return;
    }

    let html = '<div class="tools-grid">';
    tools.forEach(tool => {
        let paramsHtml = '';
        if (tool.parameters && tool.parameters.length > 0) {
            paramsHtml = '<div class="tool-parameters">';
            tool.parameters.forEach(param => {
                paramsHtml += `<div class="tool-param"><span class="tool-param-name">${param.name}</span>: ${param.description}</div>`;
            });
            paramsHtml += '</div>';
        }

        html += `
            <div class="tool-card">
                <div class="tool-name">${tool.name}</div>
                <div class="tool-description">${tool.description}</div>
                ${paramsHtml}
            </div>
        `;
    });
    html += '</div>';

    DOM.toolsContent.innerHTML = html;
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
