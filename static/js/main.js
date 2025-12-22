// ===== Main Entry Point =====
// This file loads all modules and initializes the app

function init() {
    initDOM();
    initTheme();
    initSound();
    initEventListeners();
    updateInputState();
    connect();

    // Update timestamps every minute
    setInterval(updateAllTimestamps, 60000);
}

// Start when DOM is ready
document.addEventListener('DOMContentLoaded', init);
