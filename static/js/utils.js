// ===== Utility Functions =====

function escapeHtml(s) {
    return String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

function linkify(text) {
    // Match URLs and convert to clickable links with preview
    return text.replace(/(https?:\/\/[^\s]+)/g, (url) => {
        const extension = url.split('.').pop().toLowerCase().split('?')[0];
        const isImage = IMAGE_EXTENSIONS.includes(extension);

        if (isImage) {
            return `<a href="${url}" target="_blank" rel="noopener noreferrer" class="image-link">
        <img src="${url}" alt="Image" class="image-preview" loading="lazy" onerror="this.style.display='none'">
      </a>`;
        }
        return `<a href="${url}" target="_blank" rel="noopener noreferrer" class="link-preview">🔗 ${url.length > 40 ? url.slice(0, 40) + '...' : url}</a>`;
    });
}

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

function fullTimestamp(ts) {
    const date = new Date(ts * 1000);
    return date.toLocaleString();
}

function updateAllTimestamps() {
    document.querySelectorAll('.message-time[data-ts]').forEach(el => {
        el.textContent = relativeTime(parseInt(el.dataset.ts));
    });
}

function highlightMentions(text) {
    return text.replace(/@(\w+)/g, (match, name) => {
        const isMentioned = name.toLowerCase() === myName.toLowerCase();
        const cls = isMentioned ? 'mention mention-me' : 'mention';
        return `<span class="${cls}">${match}</span>`;
    });
}

// ===== Search Functions =====

function searchMessages(query) {
    if (!query.trim()) {
        hideSearchResults();
        return;
    }

    const lowerQuery = query.toLowerCase();
    const results = allMessages.filter(msg =>
        msg.text.toLowerCase().includes(lowerQuery) ||
        msg.from.toLowerCase().includes(lowerQuery)
    );

    showSearchResults(results, query);
}

function showSearchResults(results, query) {
    if (!DOM.searchResults || !DOM.searchItems) return;

    DOM.searchResults.classList.remove('hidden');
    DOM.messagesEl.classList.add('hidden');

    if (results.length === 0) {
        DOM.searchItems.innerHTML = '<div class="no-results">No messages found</div>';
        return;
    }

    DOM.searchItems.innerHTML = results.map(msg => {
        const highlightedText = escapeHtml(msg.text).replace(
            new RegExp(`(${escapeHtml(query)})`, 'gi'),
            '<mark>$1</mark>'
        );
        return `
      <div class="search-item" data-msg-id="${msg.id}">
        <div class="search-item-header">
          <span class="search-item-author">${escapeHtml(msg.from)}</span>
          <span class="search-item-time">${relativeTime(msg.ts)}</span>
        </div>
        <div class="search-item-text">${highlightedText}</div>
      </div>
    `;
    }).join('');
}

function hideSearchResults() {
    if (!DOM.searchResults) return;
    DOM.searchResults.classList.add('hidden');
    DOM.messagesEl.classList.remove('hidden');
}

// ===== Emoji Picker =====

function buildEmojiPicker() {
    if (!DOM.emojiPicker) return;

    let html = '<div class="emoji-picker-inner">';
    for (const [category, emojis] of Object.entries(EMOJI_CATEGORIES)) {
        html += `<div class="emoji-category">
      <div class="emoji-category-title">${category}</div>
      <div class="emoji-grid">
        ${emojis.map(e => `<button class="emoji-item" data-emoji="${e}">${e}</button>`).join('')}
      </div>
    </div>`;
    }
    html += '</div>';
    DOM.emojiPicker.innerHTML = html;
}

function toggleEmojiPicker() {
    if (!DOM.emojiPicker) return;

    emojiPickerOpen = !emojiPickerOpen;
    DOM.emojiPicker.classList.toggle('open', emojiPickerOpen);

    if (emojiPickerOpen) {
        const btn = DOM.emojiBtn.getBoundingClientRect();
        DOM.emojiPicker.style.bottom = (window.innerHeight - btn.top + 8) + 'px';
        DOM.emojiPicker.style.left = btn.left + 'px';
    }
}

function closeEmojiPicker() {
    emojiPickerOpen = false;
    if (DOM.emojiPicker) DOM.emojiPicker.classList.remove('open');
}

function insertEmoji(emoji) {
    if (!DOM.textInput) return;
    const start = DOM.textInput.selectionStart;
    const end = DOM.textInput.selectionEnd;
    const text = DOM.textInput.value;
    DOM.textInput.value = text.slice(0, start) + emoji + text.slice(end);
    DOM.textInput.selectionStart = DOM.textInput.selectionEnd = start + emoji.length;
    DOM.textInput.focus();
    closeEmojiPicker();
}
