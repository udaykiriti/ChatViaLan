// ===== Configuration =====
const wsUrl = (location.protocol === 'https:' ? 'wss://' : 'ws://') + location.host + '/ws';
const commands = ['/name', '/msg', '/list', '/history', '/join', '/rooms', '/register', '/login', '/help', '/who', '/leave', '/room'];
const REACTION_EMOJIS = ['👍', '❤️', '😂', '😮', '😢', '🎉'];

// Notification sound (short beep as base64)
const NOTIFICATION_SOUND = 'data:audio/wav;base64,UklGRnoGAABXQVZFZm10IBAAAAABAAEAQB8AAEAfAAABAAgAZGF0YQoGAACBhYqFbF1peYBymXyagI+Sf5WRj5GLoZeYmJ2Xq6eTk5CRkZmPk5KRkoyTkpKYjZaUk5WRkJGUlZWOk5WVk5KPkpOVlZCSlZWTko+Sk5WVkJKVlZOSj5KTlZWQkpWVk5KPkpOVlZCSlZWTko+Sk5SVkJKVlZOSj5KTlJWQkpWVk5KPkpOUlZCSlZWTko+Sk5SVkJKVlZOSj5KTlJWQkpWVk5KPkpOUlQ==';
let notificationAudio = null;
let soundEnabled = true;

// ===== State =====
let ws = null;
let connected = false;
let named = false;
let currentRoom = 'lobby';
let typingTimeout = null;
let isTyping = false;
let unreadCount = 0;
let windowFocused = true;
let myName = '';
let lastMsgId = null;

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
const themeToggle = $('themeToggle');
const typingIndicator = $('typingIndicator');
const typingText = $('typingText');

// ===== Theme =====
function initTheme() {
    const saved = localStorage.getItem('theme') || 'dark';
    document.documentElement.setAttribute('data-theme', saved);
    updateThemeIcon();
}

function toggleTheme() {
    const current = document.documentElement.getAttribute('data-theme');
    const next = current === 'light' ? 'dark' : 'light';
    document.documentElement.setAttribute('data-theme', next);
    localStorage.setItem('theme', next);
    updateThemeIcon();
}

function updateThemeIcon() {
    const theme = document.documentElement.getAttribute('data-theme');
    themeToggle.textContent = theme === 'light' ? '☀️' : '🌙';
}

// ===== Sound Notifications =====
function initSound() {
    notificationAudio = new Audio(NOTIFICATION_SOUND);
    notificationAudio.volume = 0.3;
}

function playNotificationSound() {
    if (soundEnabled && notificationAudio && !windowFocused) {
        notificationAudio.play().catch(() => { });
    }
}

// ===== Unread Count =====
function updateUnreadCount() {
    if (unreadCount > 0) {
        document.title = `(${unreadCount}) LAN Chat`;
    } else {
        document.title = 'LAN Chat';
    }
}

function incrementUnread() {
    if (!windowFocused) {
        unreadCount++;
        updateUnreadCount();
    }
}

function clearUnread() {
    unreadCount = 0;
    updateUnreadCount();
}

// ===== Relative Time =====
function relativeTime(ts) {
    const now = Date.now();
    const diff = now - (ts * 1000);
    const mins = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);

    if (mins < 1) return 'just now';
    if (mins < 60) return `${mins}m ago`;
    if (hours < 24) return `${hours}h ago`;
    if (days < 7) return `${days}d ago`;
    return new Date(ts * 1000).toLocaleDateString();
}

function updateAllTimestamps() {
    document.querySelectorAll('.message-time[data-ts]').forEach(el => {
        el.textContent = relativeTime(parseInt(el.dataset.ts));
    });
}

// ===== Typing Indicator =====
function sendTypingStatus(typing) {
    if (!connected || !named) return;
    if (isTyping === typing) return;
    isTyping = typing;
    ws.send(JSON.stringify({ type: 'typing', is_typing: typing }));
}

function handleTypingIndicator(users) {
    const others = users.filter(u => u !== myName);
    if (others.length === 0) {
        typingIndicator.classList.remove('visible');
    } else {
        typingIndicator.classList.add('visible');
        typingText.textContent = others.length === 1
            ? `${others[0]} is typing`
            : `${others.length} people are typing`;
    }
}

// ===== Mentions =====
function highlightMentions(text) {
    return text.replace(/@(\w+)/g, (match, name) => {
        const isMentioned = name.toLowerCase() === myName.toLowerCase();
        const cls = isMentioned ? 'mention mention-me' : 'mention';
        return `<span class="${cls}">${match}</span>`;
    });
}

// ===== Reactions =====
function createReactionBar(msgId, reactions = {}) {
    let html = '<div class="reaction-bar">';

    // Existing reactions
    for (const [emoji, users] of Object.entries(reactions)) {
        if (users.length > 0) {
            const isMine = users.includes(myName);
            html += `<button class="reaction-btn ${isMine ? 'active' : ''}" data-msg-id="${msgId}" data-emoji="${emoji}">${emoji} ${users.length}</button>`;
        }
    }

    // Add reaction button
    html += `<button class="add-reaction-btn" data-msg-id="${msgId}">+</button>`;
    html += '</div>';
    return html;
}

function createReactionPicker(msgId) {
    let html = '<div class="reaction-picker">';
    REACTION_EMOJIS.forEach(emoji => {
        html += `<button class="picker-emoji" data-msg-id="${msgId}" data-emoji="${emoji}">${emoji}</button>`;
    });
    html += '</div>';
    return html;
}

// ===== Message Actions =====
function createMessageActions(msgId, from) {
    if (from !== myName) return '';
    return `<div class="message-actions">
    <button class="action-btn edit-btn" data-msg-id="${msgId}" title="Edit">✏️</button>
    <button class="action-btn delete-btn" data-msg-id="${msgId}" title="Delete">🗑️</button>
  </div>`;
}

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
                case 'msg':
                    appendMessage(data.id, data.from, data.text, data.ts, data.reactions || {}, data.edited);
                    if (data.from !== myName) {
                        playNotificationSound();
                        incrementUnread();
                    }
                    break;
                case 'list': updateUsers(data.users || []); break;
                case 'history':
                    (data.items || []).forEach(m => appendMessage(m.id, m.from, m.text, m.ts, m.reactions || {}, m.edited));
                    break;
                case 'typing': handleTypingIndicator(data.users || []); break;
                case 'reaction': handleReactionUpdate(data.msg_id, data.emoji, data.user, data.added); break;
                case 'edit': handleEditUpdate(data.msg_id, data.new_text); break;
                case 'delete': handleDeleteUpdate(data.msg_id); break;
                case 'readreceipt': handleReadReceipt(data.user, data.last_msg_id); break;
                case 'mention': handleMention(data.from, data.text, data.mentioned); break;
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

// ===== Handle Updates =====
function handleReactionUpdate(msgId, emoji, user, added) {
    const msgEl = document.querySelector(`[data-msg-id="${msgId}"]`);
    if (!msgEl) return;

    let reactionBar = msgEl.querySelector('.reaction-bar');
    if (!reactionBar) {
        const content = msgEl.querySelector('.message-content');
        content.insertAdjacentHTML('afterend', createReactionBar(msgId, {}));
        reactionBar = msgEl.querySelector('.reaction-bar');
    }

    let btn = reactionBar.querySelector(`[data-emoji="${emoji}"]`);
    if (added) {
        if (btn) {
            const count = parseInt(btn.textContent.match(/\d+/)?.[0] || '0') + 1;
            btn.textContent = `${emoji} ${count}`;
            if (user === myName) btn.classList.add('active');
        } else {
            const addBtn = reactionBar.querySelector('.add-reaction-btn');
            addBtn.insertAdjacentHTML('beforebegin',
                `<button class="reaction-btn ${user === myName ? 'active' : ''}" data-msg-id="${msgId}" data-emoji="${emoji}">${emoji} 1</button>`);
        }
    } else {
        if (btn) {
            const count = parseInt(btn.textContent.match(/\d+/)?.[0] || '1') - 1;
            if (count <= 0) {
                btn.remove();
            } else {
                btn.textContent = `${emoji} ${count}`;
                if (user === myName) btn.classList.remove('active');
            }
        }
    }
}

function handleEditUpdate(msgId, newText) {
    const msgEl = document.querySelector(`[data-msg-id="${msgId}"]`);
    if (!msgEl) return;
    const content = msgEl.querySelector('.message-content');
    content.innerHTML = linkify(highlightMentions(escapeHtml(newText)));

    let editedLabel = msgEl.querySelector('.edited-label');
    if (!editedLabel) {
        const header = msgEl.querySelector('.message-header');
        header.insertAdjacentHTML('beforeend', '<span class="edited-label">(edited)</span>');
    }
}

function handleDeleteUpdate(msgId) {
    const msgEl = document.querySelector(`[data-msg-id="${msgId}"]`);
    if (msgEl) {
        msgEl.classList.add('deleted');
        msgEl.innerHTML = '<div class="deleted-text">This message was deleted</div>';
    }
}

function handleReadReceipt(user, lastMsgId) {
    // Could show read indicators - simplified for now
}

function handleMention(from, text, mentioned) {
    if (mentioned.toLowerCase() === myName.toLowerCase()) {
        playNotificationSound();
        incrementUnread();
    }
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
        const m = text.match(/'([^']+)'/);
        if (m) myName = m[1];
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

function appendMessage(id, from, text, ts, reactions = {}, edited = false) {
    const div = document.createElement('div');
    div.className = 'message';
    div.dataset.msgId = id;

    const editedLabel = edited ? '<span class="edited-label">(edited)</span>' : '';

    div.innerHTML = `
    <div class="message-header">
      <span class="message-author">${escapeHtml(from)}</span>
      <span class="message-time" data-ts="${ts}">${relativeTime(ts)}</span>
      ${editedLabel}
      ${createMessageActions(id, from)}
    </div>
    <div class="message-content">${linkify(highlightMentions(escapeHtml(text)))}</div>
    ${createReactionBar(id, reactions)}
  `;
    messagesEl.appendChild(div);
    messagesEl.scrollTop = messagesEl.scrollHeight;

    lastMsgId = id;

    // Send read receipt
    if (windowFocused && connected) {
        ws.send(JSON.stringify({ type: 'markread', last_msg_id: id }));
    }
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
    sendTypingStatus(false);
}

function sendReaction(msgId, emoji) {
    if (!connected) return;
    ws.send(JSON.stringify({ type: 'react', msg_id: msgId, emoji }));
}

function sendEdit(msgId, newText) {
    if (!connected) return;
    ws.send(JSON.stringify({ type: 'edit', msg_id: msgId, new_text: newText }));
}

function sendDelete(msgId) {
    if (!connected) return;
    ws.send(JSON.stringify({ type: 'delete', msg_id: msgId }));
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
    // Theme toggle
    themeToggle.onclick = toggleTheme;

    // Name setting
    setNameBtn.onclick = () => {
        const n = nameInput.value.trim();
        if (!n) return;
        sendCommand('/name ' + n);
        myName = n;
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

    // Keyboard handling + typing detection
    textInput.onkeydown = (e) => {
        if (e.key === 'Enter') { sendBtn.click(); return; }
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

    // Command suggestions + typing indicator
    textInput.oninput = () => {
        const v = textInput.value.trim();
        if (v.startsWith('/')) {
            const matches = commands.filter(c => c.startsWith(v.toLowerCase()));
            if (matches.length) showCmdSuggest(matches);
            else hideCmdSuggest();
            sendTypingStatus(false);
        } else {
            hideCmdSuggest();
            if (v.length > 0) {
                sendTypingStatus(true);
                clearTimeout(typingTimeout);
                typingTimeout = setTimeout(() => sendTypingStatus(false), 2000);
            } else {
                sendTypingStatus(false);
            }
        }
    };

    // Room/User clicks
    roomList.onclick = (e) => {
        const item = e.target.closest('.room-item');
        if (item) sendCommand('/join ' + item.dataset.room);
    };

    userList.onclick = (e) => {
        const item = e.target.closest('.user-item');
        if (item) {
            textInput.value = '@' + item.dataset.user + ' ';
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
        btn.onclick = () => document.querySelector(`.auth-tab[data-target="${btn.dataset.target}"]`).click();
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

        // Close any open reaction pickers
        if (!e.target.closest('.add-reaction-btn') && !e.target.closest('.reaction-picker')) {
            document.querySelectorAll('.reaction-picker').forEach(p => p.remove());
        }
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

    // Reaction clicks (delegated)
    messagesEl.onclick = (e) => {
        const reactionBtn = e.target.closest('.reaction-btn');
        if (reactionBtn) {
            sendReaction(reactionBtn.dataset.msgId, reactionBtn.dataset.emoji);
            return;
        }

        const addReactionBtn = e.target.closest('.add-reaction-btn');
        if (addReactionBtn) {
            document.querySelectorAll('.reaction-picker').forEach(p => p.remove());
            addReactionBtn.insertAdjacentHTML('afterend', createReactionPicker(addReactionBtn.dataset.msgId));
            return;
        }

        const pickerEmoji = e.target.closest('.picker-emoji');
        if (pickerEmoji) {
            sendReaction(pickerEmoji.dataset.msgId, pickerEmoji.dataset.emoji);
            document.querySelectorAll('.reaction-picker').forEach(p => p.remove());
            return;
        }

        const editBtn = e.target.closest('.edit-btn');
        if (editBtn) {
            const msgEl = document.querySelector(`[data-msg-id="${editBtn.dataset.msgId}"]`);
            const content = msgEl.querySelector('.message-content');
            const currentText = content.textContent;
            const newText = prompt('Edit message:', currentText);
            if (newText && newText !== currentText) {
                sendEdit(editBtn.dataset.msgId, newText);
            }
            return;
        }

        const deleteBtn = e.target.closest('.delete-btn');
        if (deleteBtn) {
            if (confirm('Delete this message?')) {
                sendDelete(deleteBtn.dataset.msgId);
            }
        }
    };

    // Window focus/blur for unread count
    window.onfocus = () => {
        windowFocused = true;
        clearUnread();
        if (lastMsgId && connected) {
            ws.send(JSON.stringify({ type: 'markread', last_msg_id: lastMsgId }));
        }
    };
    window.onblur = () => { windowFocused = false; };
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
    initTheme();
    initSound();
    initEventListeners();
    updateInputState();
    connect();

    // Update timestamps every minute
    setInterval(updateAllTimestamps, 60000);
}

// Start when DOM is ready
document.addEventListener('DOMContentLoaded', init);
