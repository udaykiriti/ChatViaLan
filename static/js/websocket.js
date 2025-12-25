// ===== WebSocket Connection =====

function connect() {
    ws = new WebSocket(wsUrl);
    DOM.connStatus.textContent = 'connecting...';
    DOM.statusDot.classList.remove('connected');

    ws.onopen = () => {
        connected = true;
        DOM.connStatus.textContent = 'connected';
        DOM.statusDot.classList.add('connected');
        updateInputState();
        appendSystem('Connected to server. Set your name to start chatting.');
        sendCommand('/rooms');

        // Auto-refresh rooms every 10 seconds
        setInterval(() => {
            if (connected) sendCommand('/rooms');
        }, 10000);
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
                case 'roomlist': updateAvailableRooms(data.rooms || []); break;
                case 'history':
                    const items = data.items || [];
                    items.forEach(m => appendMessage(m.id, m.from, m.text, m.ts, m.reactions || {}, m.edited));
                    if (items.length === 0) showEmptyState();
                    break;
                case 'typing': handleTypingIndicator(data.users || []); break;
                case 'reaction': handleReactionUpdate(data.msg_id, data.emoji, data.user, data.added); break;
                case 'edit': handleEditUpdate(data.msg_id, data.new_text); break;
                case 'delete': handleDeleteUpdate(data.msg_id); break;
                case 'readreceipt': break;
                case 'linkpreview':
                    try { renderLinkPreview(data); }
                    catch (e) { console.error("Preview failed:", e); }
                    break;
                case 'status':
                    userStatuses[data.user] = data.status;
                    sendCommand('/list');
                    break;
                case 'mention':
                    if (data.mentioned.toLowerCase() === myName.toLowerCase()) {
                        playNotificationSound();
                        incrementUnread();
                    }
                    break;
                default: handleSystem(e.data);
            }
        } catch (err) {
            console.error("WS Error:", err);
            handleSystem(e.data);
        }
    };

    ws.onclose = () => {
        connected = false;
        DOM.connStatus.textContent = 'disconnected';
        DOM.statusDot.classList.remove('connected');
        updateInputState();
        appendSystem('Disconnected. Reconnecting...');
        setTimeout(connect, 2000);
    };

    ws.onerror = () => {
        DOM.connStatus.textContent = 'error';
        DOM.statusDot.classList.remove('connected');
        // Ensure we try to reconnect if error doesn't trigger close
        if (ws.readyState === WebSocket.CLOSED || ws.readyState === WebSocket.CLOSING) {
            // onclose will handle it
        } else {
            ws.close(); // Force close to trigger onclose reconnection
        }
    };
}

// ===== System Message Handler =====

function handleSystem(text) {
    appendSystem(text);

    if (text.includes("You joined room") || text.includes("joined the room")) {
        const m = text.match(/room '([^']+)'/);
        if (m) {
            currentRoom = m[1];
            DOM.roomBadge.textContent = '#' + currentRoom;
            sendCommand('/rooms');
        }
        sendCommand('/list');
    }

    if (text.includes("Your name is") || text.includes("Logged in") || text.includes("Register failed") || text.includes("Login failed")) {
        const m = text.match(/'([^']+)'/);
        if (m) myName = m[1];
        named = true;

        // Reset auth buttons
        const loginBtn = $('loginBtn');
        const regBtn = $('regBtn');
        if (loginBtn) {
            loginBtn.classList.remove('loading');
            loginBtn.textContent = 'Login';
        }
        if (regBtn) {
            regBtn.classList.remove('loading');
            regBtn.textContent = 'Register';
        }

        updateInputState();
        sendCommand('/list');
        sendCommand('/rooms');
    }
}

// ===== Send Functions =====

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

function sendTypingStatus(isTyping) {
    if (!connected) return;
    // Don't send if state hasn't changed (optimization)
    if (window.lastTypingState === isTyping) return;
    window.lastTypingState = isTyping;

    ws.send(JSON.stringify({ type: 'typing', is_typing: isTyping }));
}
