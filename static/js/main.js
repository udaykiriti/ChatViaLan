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

    // Register Service Worker for PWA (lower priority)
    // NUCLEAR CACHE CLEAR: Unregister all service workers to force update
    if ('serviceWorker' in navigator) {
        navigator.serviceWorker.getRegistrations().then(function (registrations) {
            for (let registration of registrations) {
                registration.unregister();
            }
        });
    }

    // Update timestamps every minute
    setInterval(updateAllTimestamps, 60000);
}

// Start when DOM is ready
document.addEventListener('DOMContentLoaded', init);
