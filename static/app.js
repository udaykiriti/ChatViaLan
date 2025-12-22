// ===== Configuration =====
const wsUrl = (location.protocol === 'https:' ? 'wss://' : 'ws://') + location.host + '/ws';
const commands = ['/name', '/msg', '/list', '/history', '/join', '/rooms', '/register', '/login', '/help', '/who', '/leave', '/room'];

// ===== State =====
let ws = null;
let connected = false;
let named = false;
let currentRoom = 'lobby';

// ===== DOM Elements =====
const $ = id => document.getElementById(id);
const statusDot = $('statusDot');
const connStatus = $('connectionStatus');
const roomBadge = $('currentRoom');
const nameInput = $('nameInput');
const setNameBtn = $('setNameBtn');
const messagesEl = $('messages');
const textInput = $('textInput');
const sendBtn = $('sendBtn');
const fileInput = $('fileInput');
const uploadBtn = $('uploadBtn');
const uploadMsg = $('uploadMsg');
const userList = $('userList');
const roomList = $('roomList');
const newRoomInput = $('newRoomInput');
const joinRoomBtn = $('joinRoomBtn');
const authMsg = $('authMsg');
const cmdSuggest = $('cmdSuggest');

// ===== WebSocket Connection =====
function connect() {
    ws = new WebSocket(wsUrl);
    connStatus.textContent = 'connecting...';
    statusDot.classList.remove('connected');

    ws.onopen = () => {
        connected = true;
        connStatus.textContent = 'connected';
        statusDot.classList.add('connected');
        updateInputState();
        appendSystem('Connected to server. Set your name to start chatting.');
        sendCommand('/rooms');
    };

    ws.onmessage = (e) => {
        try {
            const data = JSON.parse(e.data);
            switch (data.type) {
                case 'system': handleSystem(data.text); break;
                case 'msg': appendMessage(data.from, data.text, data.ts); break;
                case 'list': updateUsers(data.users || []); break;
                case 'history': (data.items || []).forEach(m => appendMessage(m.from, m.text, m.ts)); break;
                default: handleSystem(e.data);
            }
        } catch { handleSystem(e.data); }
    };

    ws.onclose = () => {
        connected = false;
        connStatus.textContent = 'disconnected';
        statusDot.classList.remove('connected');
        updateInputState();
        appendSystem('Disconnected. Reconnecting...');
        setTimeout(connect, 2000);
    };

    ws.onerror = () => {
        connStatus.textContent = 'error';
        statusDot.classList.remove('connected');
    };
}

// ===== Message Handling =====
function handleSystem(text) {
    appendSystem(text);

    if (text.startsWith('Rooms:')) {
        const rooms = text.slice(6).trim().split(',').map(r => r.trim()).filter(Boolean);
        updateRooms(rooms);
    }

    if (text.includes("You joined room") || text.includes("joined the room")) {
        const m = text.match(/room '([^']+)'/);
        if (m) {
            currentRoom = m[1];
            roomBadge.textContent = '#' + currentRoom;
        }
        sendCommand('/list');
    }

    if (text.includes("Your name is") || text.includes("Logged in")) {
        named = true;
        updateInputState();
        sendCommand('/list');
    }
}

function appendSystem(text) {
    const div = document.createElement('div');
    div.className = 'message system';
    div.textContent = text;
    messagesEl.appendChild(div);
    messagesEl.scrollTop = messagesEl.scrollHeight;
}

function appendMessage(from, text, ts) {
    const div = document.createElement('div');
    div.className = 'message';
    const time = new Date(ts * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    div.innerHTML = `
    <div class="message-header">
      <span class="message-author">${escapeHtml(from)}</span>
      <span class="message-time">${time}</span>
    </div>
    <div class="message-content">${linkify(escapeHtml(text))}</div>
  `;
    messagesEl.appendChild(div);
    messagesEl.scrollTop = messagesEl.scrollHeight;
}

function escapeHtml(s) {
    return String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

function linkify(text) {
    return text.replace(/(https?:\/\/[^\s]+)/g, '<a href="$1" target="_blank">$1</a>');
}

// ===== User/Room Lists =====
function updateUsers(users) {
    userList.innerHTML = users.map(u => `
    <li class="user-item" data-user="${escapeHtml(u)}">
      <div class="user-avatar">${u[0].toUpperCase()}</div>
      <span>${escapeHtml(u)}</span>
    </li>
  `).join('');
}

function updateRooms(rooms) {
    roomList.innerHTML = rooms.map(r => `
    <li class="room-item" data-room="${escapeHtml(r)}">
      <span class="room-icon">#</span>
      <span>${escapeHtml(r)}</span>
    </li>
  `).join('');
}

// ===== Commands =====
function sendCommand(cmd) {
    if (!connected) return appendSystem('Not connected.');
    ws.send(JSON.stringify({ type: 'cmd', cmd }));
}

function sendMessage(msg) {
    if (!connected) return appendSystem('Not connected.');
    ws.send(JSON.stringify({ type: 'msg', text: msg }));
}

// ===== Input State =====
function updateInputState() {
    const disabled = !connected || !named;
    textInput.disabled = disabled;
    sendBtn.disabled = disabled;
    uploadBtn.disabled = disabled;
    nameInput.disabled = !connected;
    setNameBtn.disabled = !connected;
}

// ===== Event Listeners =====
function initEventListeners() {
    // Name setting
    setNameBtn.onclick = () => {
        const n = nameInput.value.trim();
        if (!n) return;
        sendCommand('/name ' + n);
        named = true;
        updateInputState();
    };

    // Send message
    sendBtn.onclick = () => {
        const t = textInput.value.trim();
        if (!t) return;
        t.startsWith('/') ? sendCommand(t) : sendMessage(t);
        textInput.value = '';
        hideCmdSuggest();
    };

    // Keyboard handling
    textInput.onkeydown = (e) => {
        if (e.key === 'Enter') sendBtn.click();
        if (cmdSuggest.style.display === 'block') {
            const items = [...cmdSuggest.querySelectorAll('li')];
            const active = cmdSuggest.querySelector('.active');
            let idx = items.indexOf(active);
            if (e.key === 'ArrowDown') { e.preventDefault(); idx = (idx + 1) % items.length; }
            else if (e.key === 'ArrowUp') { e.preventDefault(); idx = (idx - 1 + items.length) % items.length; }
            else if (e.key === 'Tab' && active) { e.preventDefault(); textInput.value = active.textContent + ' '; hideCmdSuggest(); return; }
            items.forEach(i => i.classList.remove('active'));
            if (items[idx]) items[idx].classList.add('active');
        }
    };

    // Command suggestions
    textInput.oninput = () => {
        const v = textInput.value.trim();
        if (v.startsWith('/')) {
            const matches = commands.filter(c => c.startsWith(v.toLowerCase()));
            if (matches.length) showCmdSuggest(matches);
            else hideCmdSuggest();
        } else hideCmdSuggest();
    };

    // Room/User clicks
    roomList.onclick = (e) => {
        const item = e.target.closest('.room-item');
        if (item) sendCommand('/join ' + item.dataset.room);
    };

    userList.onclick = (e) => {
        const item = e.target.closest('.user-item');
        if (item) {
            textInput.value = '/msg ' + item.dataset.user + ' ';
            textInput.focus();
        }
    };

    joinRoomBtn.onclick = () => {
        const r = newRoomInput.value.trim();
        if (r) { sendCommand('/join ' + r); newRoomInput.value = ''; }
    };

    // Auth tabs
    document.querySelectorAll('.auth-tab').forEach(tab => {
        tab.onclick = () => {
            document.querySelectorAll('.auth-tab').forEach(t => t.classList.remove('active'));
            document.querySelectorAll('.auth-content').forEach(c => c.classList.remove('active'));
            tab.classList.add('active');
            $(tab.dataset.tab + 'Tab').classList.add('active');
        };
    });

    document.querySelectorAll('.switch-tab').forEach(btn => {
        btn.onclick = () => document.querySelector(`.auth-tab[data-tab="${btn.dataset.target}"]`).click();
    });

    // Login/Register
    $('loginBtn').onclick = () => {
        const u = $('loginUser').value.trim(), p = $('loginPass').value;
        if (!u || !p) return showAuthMsg('Fill all fields', true);
        sendCommand(`/login ${u} ${p}`);
    };

    $('regBtn').onclick = () => {
        const u = $('regUser').value.trim(), p = $('regPass').value;
        if (!u || !p) return showAuthMsg('Fill all fields', true);
        sendCommand(`/register ${u} ${p}`);
    };

    // File upload
    uploadBtn.onclick = async () => {
        if (!fileInput.files?.length) return uploadMsg.textContent = 'Select a file first';
        const fd = new FormData();
        fd.append('file', fileInput.files[0]);
        uploadBtn.disabled = true;
        uploadMsg.textContent = 'Uploading...';
        try {
            const res = await fetch('/upload', { method: 'POST', body: fd });
            const j = await res.json();
            if (j.ok && j.files?.length) {
                j.files.forEach(f => sendMessage(`📎 ${f.filename}: ${location.origin}${f.url}`));
                uploadMsg.textContent = 'Done!';
                fileInput.value = '';
            } else uploadMsg.textContent = 'Upload failed';
        } catch (e) { uploadMsg.textContent = 'Error: ' + e; }
        uploadBtn.disabled = false;
        setTimeout(() => uploadMsg.textContent = '', 3000);
    };

    // Close suggestions on outside click
    document.onclick = (e) => {
        if (!cmdSuggest.contains(e.target) && e.target !== textInput) hideCmdSuggest();
    };

    // Escape to close
    document.onkeydown = (e) => { if (e.key === 'Escape') hideCmdSuggest(); };

    // Command suggestion clicks
    cmdSuggest.onclick = (e) => {
        if (e.target.tagName === 'LI') {
            textInput.value = e.target.textContent + ' ';
            hideCmdSuggest();
            textInput.focus();
        }
    };
}

// ===== Command Suggestions UI =====
function showCmdSuggest(items) {
    cmdSuggest.innerHTML = items.map((c, i) => `<li class="${i === 0 ? 'active' : ''}">${c}</li>`).join('');
    const r = textInput.getBoundingClientRect();
    cmdSuggest.style.left = r.left + 'px';
    cmdSuggest.style.bottom = (window.innerHeight - r.top + 4) + 'px';
    cmdSuggest.style.width = r.width + 'px';
    cmdSuggest.style.display = 'block';
}

function hideCmdSuggest() {
    cmdSuggest.style.display = 'none';
}

// ===== Auth Message =====
function showAuthMsg(msg, isError = false) {
    authMsg.textContent = msg;
    authMsg.style.display = 'block';
    authMsg.style.background = isError ? 'rgba(239,68,68,0.2)' : 'rgba(34,197,94,0.2)';
    authMsg.style.color = isError ? 'var(--danger)' : 'var(--success)';
    setTimeout(() => authMsg.style.display = 'none', 3000);
}

// ===== Initialize App =====
function init() {
    initEventListeners();
    updateInputState();
    connect();
}

// Start when DOM is ready
document.addEventListener('DOMContentLoaded', init);
