// ===== Reaction Functions =====

function createReactionBar(msgId, reactions = {}) {
    let html = '<div class="reaction-bar">';

    for (const [emoji, users] of Object.entries(reactions)) {
        if (users.length > 0) {
            const isMine = users.includes(myName);
            html += `<button class="reaction-btn ${isMine ? 'active' : ''}" data-msg-id="${msgId}" data-emoji="${emoji}">${emoji} ${users.length}</button>`;
        }
    }

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

function createMessageActions(msgId, from) {
    let html = '<div class="message-actions">';

    // Reply button (everyone can reply)
    html += `<button class="action-btn reply-btn" data-msg-id="${msgId}" title="Reply">↩️</button>`;

    // Edit/Delete only for own messages
    if (from === myName) {
        html += `<button class="action-btn edit-btn" data-msg-id="${msgId}" title="Edit">✏️</button>`;
        html += `<button class="action-btn delete-btn" data-msg-id="${msgId}" title="Delete">🗑️</button>`;
    }

    // Pin button
    html += `<button class="action-btn pin-btn" data-msg-id="${msgId}" title="Pin">📌</button>`;

    html += '</div>';
    return html;
}

function handleReactionUpdate(msgId, emoji, user, added) {
    const msgEl = document.querySelector(`[data-msg-id="${msgId}"]`);
    if (!msgEl) return;

    let reactionBar = msgEl.querySelector('.reaction-bar');
    if (!reactionBar) {
        const bubble = msgEl.querySelector('.message-bubble');
        if (bubble) {
            bubble.insertAdjacentHTML('afterend', createReactionBar(msgId, {}));
            reactionBar = msgEl.querySelector('.reaction-bar');
        }
    }

    if (!reactionBar) return;

    let btn = reactionBar.querySelector(`[data-emoji="${emoji}"]`);
    if (added) {
        if (btn) {
            const count = parseInt(btn.textContent.match(/\d+/)?.[0] || '0') + 1;
            btn.textContent = `${emoji} ${count}`;
            if (user === myName) btn.classList.add('active');
        } else {
            const addBtn = reactionBar.querySelector('.add-reaction-btn');
            if (addBtn) {
                addBtn.insertAdjacentHTML('beforebegin',
                    `<button class="reaction-btn ${user === myName ? 'active' : ''}" data-msg-id="${msgId}" data-emoji="${emoji}">${emoji} 1</button>`);
            }
        }
    } else {
        if (btn) {
            const count = parseInt(btn.textContent.match(/\d+/)?.[0] || '1') - 1;
            if (count <= 0) btn.remove();
            else {
                btn.textContent = `${emoji} ${count}`;
                if (user === myName) btn.classList.remove('active');
            }
        }
    }
}

function handleEditUpdate(msgId, newText) {
    const msgEl = document.querySelector(`[data-msg-id="${msgId}"]`);
    if (!msgEl) return;
    const bubble = msgEl.querySelector('.message-bubble');
    if (bubble) {
        bubble.innerHTML = linkify(highlightMentions(escapeHtml(newText)));
    }

    // Update in allMessages
    const msg = allMessages.find(m => m.id === msgId);
    if (msg) msg.text = newText;

    let editedLabel = msgEl.querySelector('.edited-label');
    if (!editedLabel) {
        const header = msgEl.querySelector('.message-header');
        if (header) {
            header.insertAdjacentHTML('beforeend', '<span class="edited-label">(edited)</span>');
        }
    }
}

function handleDeleteUpdate(msgId) {
    const msgEl = document.querySelector(`[data-msg-id="${msgId}"]`);
    if (msgEl) {
        msgEl.classList.add('deleted');
        msgEl.innerHTML = '<div class="message-bubble"><span class="deleted-text">This message was deleted</span></div>';
    }

    // Remove from allMessages
    const idx = allMessages.findIndex(m => m.id === msgId);
    if (idx >= 0) allMessages.splice(idx, 1);
}
