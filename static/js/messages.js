// ===== Message Rendering =====

function appendSystem(text) {
  const div = document.createElement('div');
  div.className = 'message system';
  div.innerHTML = '<div class="message-bubble">' + escapeHtml(text) + '</div>';
  DOM.messagesEl.appendChild(div);
  DOM.messagesEl.scrollTop = DOM.messagesEl.scrollHeight;
}

function appendMessage(id, from, text, ts, reactions = {}, edited = false) {
  const div = document.createElement('div');
  const isMine = from === myName;
  div.className = `message ${isMine ? 'sent' : 'received'}`;
  div.dataset.msgId = id;

  const editedLabel = edited ? '<span class="edited-label">(edited)</span>' : '';

  div.innerHTML = `
    <div class="message-header">
      <span class="message-author">${escapeHtml(from)}</span>
      <span class="message-time" data-ts="${ts}">${relativeTime(ts)}</span>
      ${editedLabel}
      ${createMessageActions(id, from)}
    </div>
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
  DOM.userCount.textContent = `(${users.length})`;
  DOM.userList.innerHTML = users.map(u => `
    <li class="user-item" data-user="${escapeHtml(u)}">
      <div class="user-avatar">${u[0].toUpperCase()}</div>
      <span>${escapeHtml(u)}</span>
    </li>
  `).join('');
}

function updateRooms(rooms) {
  DOM.roomList.innerHTML = rooms.map(r => `
    <li class="room-item ${r === currentRoom ? 'active' : ''}" data-room="${escapeHtml(r)}">
      <div class="room-icon">#</div>
      <div class="room-info">
        <div class="room-name">${escapeHtml(r)}</div>
      </div>
    </li>
  `).join('');
}
