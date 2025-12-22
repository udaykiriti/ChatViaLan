// ===== Message Rendering =====

function appendSystem(text) {
  const div = document.createElement('div');
  div.className = 'message system';
  div.innerHTML = '<div class="message-bubble">' + escapeHtml(text) + '</div>';
  DOM.messagesEl.appendChild(div);
  DOM.messagesEl.scrollTop = DOM.messagesEl.scrollHeight;
}

function appendMessage(id, from, text, ts, reactions = {}, edited = false, replyTo = null) {
  // Store for search
  allMessages.push({ id, from, text, ts });

  const div = document.createElement('div');
  const isMine = from === myName;
  div.className = `message ${isMine ? 'sent' : 'received'}`;
  div.dataset.msgId = id;

  const editedLabel = edited ? '<span class="edited-label">(edited)</span>' : '';
  const fullDate = fullTimestamp(ts);

  // Reply preview
  let replyHtml = '';
  if (replyTo) {
    const replyMsg = allMessages.find(m => m.id === replyTo);
    if (replyMsg) {
      replyHtml = `<div class="reply-preview">
        <span class="reply-author">${escapeHtml(replyMsg.from)}</span>
        <span class="reply-text">${escapeHtml(replyMsg.text.slice(0, 50))}${replyMsg.text.length > 50 ? '...' : ''}</span>
      </div>`;
    }
  }

  div.innerHTML = `
    <div class="message-header">
      <span class="message-author">${escapeHtml(from)}</span>
      <span class="message-time" data-ts="${ts}" title="${fullDate}">${relativeTime(ts)}</span>
      ${editedLabel}
      ${createMessageActions(id, from)}
    </div>
    ${replyHtml}
    <div class="message-bubble">${linkify(highlightMentions(escapeHtml(text)))}</div>
    ${createReactionBar(id, reactions)}
  `;
  DOM.messagesEl.appendChild(div);
  DOM.messagesEl.scrollTop = DOM.messagesEl.scrollHeight;

  lastMsgId = id;

  if (windowFocused && connected) {
    ws.send(JSON.stringify({ type: 'markread', last_msg_id: id }));
  }
}

function updateUsers(users) {
  if (DOM.userCount) DOM.userCount.textContent = `(${users.length})`;
  if (DOM.userList) {
    DOM.userList.innerHTML = users.map(u => `
      <li class="user-item" data-user="${escapeHtml(u)}">
        <div class="user-avatar">${u[0].toUpperCase()}</div>
        <span>${escapeHtml(u)}</span>
        <button class="dm-btn" data-user="${escapeHtml(u)}" title="Send DM">💬</button>
      </li>
    `).join('');
  }
}

function updateAvailableRooms(rooms) {
  if (!DOM.roomList) return;

  rooms.sort((a, b) => {
    if (a.name === currentRoom) return -1;
    if (b.name === currentRoom) return 1;
    return b.members - a.members;
  });

  DOM.roomList.innerHTML = rooms.map(r => `
    <li class="room-item ${r.name === currentRoom ? 'active' : ''}" data-room="${escapeHtml(r.name)}">
      <div class="room-icon">#</div>
      <div class="room-info">
        <div class="room-name">${escapeHtml(r.name)}</div>
        <div class="room-members">${r.members} ${r.members === 1 ? 'member' : 'members'}</div>
      </div>
    </li>
  `).join('');
}

function updatePinnedMessages(messages) {
  if (!DOM.pinnedMessages) return;

  if (messages.length === 0) {
    DOM.pinnedMessages.classList.add('hidden');
    return;
  }

  DOM.pinnedMessages.classList.remove('hidden');
  DOM.pinnedMessages.innerHTML = `
    <div class="pinned-header">📌 Pinned Messages</div>
    ${messages.map(m => `
      <div class="pinned-item" data-msg-id="${m.id}">
        <span class="pinned-author">${escapeHtml(m.from)}:</span>
        <span class="pinned-text">${escapeHtml(m.text.slice(0, 60))}${m.text.length > 60 ? '...' : ''}</span>
      </div>
    `).join('')}
  `;
}
