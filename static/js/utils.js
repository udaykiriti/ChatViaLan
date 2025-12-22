// ===== Utility Functions =====

function escapeHtml(s) {
    return String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

function linkify(text) {
    return text.replace(/(https?:\/\/[^\s]+)/g, '<a href="$1" target="_blank">$1</a>');
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
