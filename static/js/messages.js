function appendSystem(text) {
  hideEmptyState();
  const div = document.createElement('div');
  div.className = 'message system';
  div.innerHTML = '<div class="message-bubble">' + escapeHtml(text) + '</div>';
  DOM.messagesEl.appendChild(div);
  DOM.messagesEl.scrollTop = DOM.messagesEl.scrollHeight;
}

function showEmptyState() {
  if (!DOM.messagesEl) return;
  if (DOM.messagesEl.querySelector('.empty-state')) return;
  if (DOM.messagesEl.children.length > 0) return;

  DOM.messagesEl.innerHTML = `
    <div class="empty-state">
      <div class="empty-state-icon">💬</div>
      <div class="empty-state-title">No messages yet</div>
      <div class="empty-state-text">Start the conversation! Send a message or join another room.</div>
    </div>
  `;
}

function hideEmptyState() {
  if (!DOM.messagesEl) return;
  const emptyState = DOM.messagesEl.querySelector('.empty-state');
  if (emptyState) emptyState.remove();
}

function appendMessage(id, from, text, ts, reactions = {}, edited = false, replyTo = null) {
  hideEmptyState(); // Hide empty state when messages arrive
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
      replyHtml = `<div class="reply-preview" onclick="jumpToMessage('${replyTo}')" style="cursor:pointer">
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
  // Check if user is scrolled to bottom before appending
  const isScrolledToBottom = DOM.messagesEl.scrollHeight - DOM.messagesEl.scrollTop <= DOM.messagesEl.clientHeight + 100;

  DOM.messagesEl.appendChild(div);

  // Auto-scroll only if we were already at the bottom OR if we sent the message
  if (isScrolledToBottom || isMine) {
    DOM.messagesEl.scrollTop = DOM.messagesEl.scrollHeight;
  }

  lastMsgId = id;

  if (windowFocused && connected) {
    ws.send(JSON.stringify({ type: 'markread', last_msg_id: id }));
  }
}

function updateUsers(users) {
  if (DOM.userCount) DOM.userCount.textContent = `(${users.length})`;
  if (DOM.userList) {
    DOM.userList.innerHTML = users.map(u => {
      const status = userStatuses[u] || 'active';
      const statusClass = status === 'idle' ? 'status-idle' : 'status-active';
      return `
        <li class="user-item" data-user="${escapeHtml(u)}">
          <div class="user-avatar">${u[0].toUpperCase()}<span class="status-dot ${statusClass}"></span></div>
          <span>${escapeHtml(u)}</span>
          <button class="dm-btn" data-user="${escapeHtml(u)}" title="Send DM">💬</button>
        </li>
      `;
    }).join('');
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
      <div class="pinned-item" data-msg-id="${m.id}" onclick="jumpToMessage('${m.id}')">
        <span class="pinned-author">${escapeHtml(m.from)}:</span>
        <span class="pinned-text">${escapeHtml(m.text.slice(0, 60))}${m.text.length > 60 ? '...' : ''}</span>
      </div>
    `).join('')}
  `;
}

function jumpToMessage(id) {
  const el = document.querySelector(`.message[data-msg-id="${id}"]`);
  if (el) {
    el.scrollIntoView({ behavior: 'smooth', block: 'center' });
    el.classList.add('highlight');
    setTimeout(() => el.classList.remove('highlight'), 2000);
  } else {
    appendSystem('Message not found in current view.');
  }
}

// Consolidated Link Preview Renderer
function renderLinkPreview(data) {
  const { msg_id, title, description, image, url } = data;

  const msgEl = document.querySelector(`.message[data-msg-id="${msg_id}"]`);
  if (!msgEl) return;
  if (msgEl.querySelector('.link-preview')) return;

  // Handle protocol-relative URLs
  let imgUrl = image;
  if (imgUrl && imgUrl.startsWith('//')) {
    imgUrl = 'https:' + imgUrl;
  }

  // Use img tag with onerror handler to hide broken images
  // We use inline onerror to set display:none if load fails
  const imgHtml = imgUrl
    ? `<img src="${escapeHtml(imgUrl)}" class="preview-image" alt="" onerror="this.style.display='none'">`
    : '';

  const previewHtml = `
    <a href="${escapeHtml(url)}" target="_blank" class="link-preview">
      ${imgHtml}
      <div class="preview-content">
        <div class="preview-title">${escapeHtml(title || url)}</div>
        <div class="preview-desc">${escapeHtml(description)}</div>
        <div class="preview-domain">${escapeHtml(new URL(url).hostname)}</div>
      </div>
    </a>
  `;

  const bubble = msgEl.querySelector('.message-bubble');
  if (bubble) {
    bubble.insertAdjacentHTML('afterend', previewHtml);
    if (DOM.messagesEl) DOM.messagesEl.scrollTop = DOM.messagesEl.scrollHeight;
  }
}
