// ===== Main Entry Point =====

function init() {
    // Initialize DOM references first!
    initDOM();

    // Initialize features
    initSound();
    initMobileMenu();
    buildEmojiPicker();
    initEventListeners();
    updateInputState();
    connect();

    // Register service worker for PWA support
    if ('serviceWorker' in navigator) {
        navigator.serviceWorker.register('/sw.js').catch(function (err) {
            console.warn('Service worker registration failed:', err);
        });
    }

    // Update timestamps every minute
    setInterval(updateAllTimestamps, 60000);
}

// Start when DOM is ready
document.addEventListener('DOMContentLoaded', init);
