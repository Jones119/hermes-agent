
let currentSessionId = null;
let messages = [];

const API_BASE = '';

// DOM elements
const chatContainer = document.getElementById('chat-container');
const welcomeScreen = document.getElementById('welcome-screen');
const chatInput = document.getElementById('chat-input');
const sendBtn = document.getElementById('send-btn');
const newChatBtn = document.getElementById('new-chat-btn');
const sessionsList = document.getElementById('sessions-list');

// Initialize
async function init() {
    await loadSessions();
    setupEventListeners();
}

function setupEventListeners() {
    newChatBtn.addEventListener('click', createNewSession);
    sendBtn.addEventListener('click', sendMessage);
    
    chatInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            sendMessage();
        }
    });
    
    document.querySelectorAll('.quick-action').forEach(btn => {
        btn.addEventListener('click', () => {
            const prompt = btn.dataset.prompt;
            createNewSession().then(() => {
                chatInput.value = prompt;
                sendMessage();
            });
        });
    });
    
    chatInput.addEventListener('input', () => {
        chatInput.style.height = 'auto';
        chatInput.style.height = chatInput.scrollHeight + 'px';
    });
}

async function createNewSession() {
    try {
        const response = await fetch(`${API_BASE}/sessions`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
        });
        
        if (response.ok) {
            const data = await response.json();
            currentSessionId = data.session_id;
            messages = [];
            renderMessages();
            await loadSessions();
            welcomeScreen.style.display = 'none';
        }
    } catch (error) {
        console.error('Failed to create session:', error);
    }
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
    sessionsList.innerHTML = '';
    sessions.forEach(sessionId => {
        const item = document.createElement('div');
        item.className = `session-item ${sessionId === currentSessionId ? 'active' : ''}`;
        item.textContent = sessionId.substring(0, 8) + '...';
        item.addEventListener('click', () => {
            currentSessionId = sessionId;
            renderSessions(sessions);
        });
        sessionsList.appendChild(item);
    });
}

async function sendMessage() {
    const input = chatInput.value.trim();
    if (!input || !currentSessionId) {
        if (!currentSessionId) {
            await createNewSession();
        }
        if (!input) return;
    }
    
    chatInput.value = '';
    chatInput.style.height = 'auto';
    sendBtn.disabled = true;
    
    // Add user message
    messages.push({ role: 'user', content: input });
    renderMessages();
    
    try {
        const response = await fetch(`${API_BASE}/sessions/${currentSessionId}/chat`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ input }),
        });
        
        if (response.ok) {
            const data = await response.json();
            messages.push({ role: 'assistant', content: data.output });
        } else {
            const error = await response.json();
            messages.push({ role: 'assistant', content: `Error: ${error.error}` });
        }
    } catch (error) {
        messages.push({ role: 'assistant', content: `Error: ${error.message}` });
    }
    
    sendBtn.disabled = false;
    renderMessages();
}

function renderMessages() {
    if (messages.length === 0) {
        welcomeScreen.style.display = 'block';
        return;
    }
    
    welcomeScreen.style.display = 'none';
    
    const existingMessages = chatContainer.querySelectorAll('.message');
    existingMessages.forEach(el => el.remove());
    
    messages.forEach(msg => {
        const messageEl = document.createElement('div');
        messageEl.className = `message ${msg.role}`;
        
        messageEl.innerHTML = `
            <div class="message-header">
                <div class="message-avatar">${msg.role === 'user' ? 'U' : 'A'}</div>
                <div class="message-role">${msg.role === 'user' ? 'You' : 'Hermes'}</div>
            </div>
            <div class="message-content">${escapeHtml(msg.content)}</div>
        `;
        
        chatContainer.appendChild(messageEl);
    });
    
    chatContainer.scrollTop = chatContainer.scrollHeight;
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// Health check on load
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
