// ===== Theme Management =====
// (Removed as part of cleanup)


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

// ===== Nudge Feature =====

function handleNudge(from) {
    // 1. Shake the App
    const app = document.querySelector('.app');
    if (app) {
        app.classList.remove('shake');
        void app.offsetWidth; // Trigger reflow
        app.classList.add('shake');
        setTimeout(() => app.classList.remove('shake'), 500);
    }

    // 2. Play Nudge Sound (Synthesized "Buzz")
    if (soundEnabled) {
        try {
            const AudioContext = window.AudioContext || window.webkitAudioContext;
            if (AudioContext) {
                const ctx = new AudioContext();
                const osc = ctx.createOscillator();
                const gain = ctx.createGain();

                osc.type = 'sawtooth';
                osc.frequency.setValueAtTime(150, ctx.currentTime);
                osc.frequency.exponentialRampToValueAtTime(100, ctx.currentTime + 0.15);

                gain.gain.setValueAtTime(0.5, ctx.currentTime);
                gain.gain.exponentialRampToValueAtTime(0.01, ctx.currentTime + 0.15);

                osc.connect(gain);
                gain.connect(ctx.destination);

                osc.start();
                osc.stop(ctx.currentTime + 0.2);
            } else {
                // Fallback
                playNotificationSound();
            }
        } catch (e) {
            console.error('Audio error:', e);
        }
    }
}
