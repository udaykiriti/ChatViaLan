// ===== Theme Management =====

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
    DOM.themeToggle.textContent = theme === 'light' ? '☀️' : '🌙';
}

// ===== Sound & Notifications =====

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
    document.title = unreadCount > 0 ? `(${unreadCount}) LAN Chat` : 'LAN Chat';
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
        DOM.typingIndicator.classList.remove('visible');
    } else {
        DOM.typingIndicator.classList.add('visible');
        DOM.typingText.textContent = others.length === 1
            ? `${others[0]} is typing`
            : `${others.length} people are typing`;
    }
}

// ===== Mobile Menu =====

function initMobileMenu() {
    if (DOM.menuBtn) {
        DOM.menuBtn.onclick = () => {
            DOM.sidebar.classList.add('active');
            DOM.sidebarOverlay.classList.add('active');
        };
    }

    if (DOM.closeSidebar) {
        DOM.closeSidebar.onclick = closeSidebar;
    }

    if (DOM.sidebarOverlay) {
        DOM.sidebarOverlay.onclick = closeSidebar;
    }
}

function closeSidebar() {
    if (DOM.sidebar) DOM.sidebar.classList.remove('active');
    if (DOM.sidebarOverlay) DOM.sidebarOverlay.classList.remove('active');
}
