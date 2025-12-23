// ===== Main Entry Point =====

function init() {
    // Initialize DOM references first!
    initDOM();

    // Initialize features
    initTheme();
    initSound();
    initMobileMenu();
    buildEmojiPicker();
    initEventListeners();
    updateInputState();
    connect();

    // Register Service Worker for PWA (lower priority)
    if ('serviceWorker' in navigator) {
        navigator.serviceWorker.register('/sw.js').catch(() => { });
    }

    // Update timestamps every minute
    setInterval(updateAllTimestamps, 60000);
}

// Start when DOM is ready
document.addEventListener('DOMContentLoaded', init);
