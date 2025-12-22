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
                case 'readreceipt': break; // Future: show read indicators
                case 'mention':
                    if (data.mentioned.toLowerCase() === myName.toLowerCase()) {
                        playNotificationSound();
                        incrementUnread();
                    }
                    break;
                default: handleSystem(e.data);
            }
        } catch { handleSystem(e.data); }
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
    };
}

// ===== System Message Handler =====

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
            DOM.roomBadge.textContent = '#' + currentRoom;
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
